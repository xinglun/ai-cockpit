use cockpit_core::Digest;
use cockpit_protocol::{
    CONTRACT_AMENDMENT_SCHEMA_VERSION, ContractAmendmentChange, ContractAmendmentOperation,
    ContractAmendmentRequest, PROTOCOL_VERSION, RuntimeContext,
};
use cockpit_repository::{
    WorkItemStartOptions, attach, record_recovery_decision,
    record_work_item_governance_controls_with_runtime, repository_id, start_work_item_with_options,
};
use serde_json::{Value, json};
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

const WORK_ITEM_ID: &str = "WI-CLI-AMEND-PARITY";

fn git(root: &Path, args: &[&str]) {
    let output = git_output(root, args);
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(root: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .args([
            "-c",
            "maintenance.autoDetach=false",
            "-c",
            "gc.autoDetach=false",
        ])
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixturePermissions {
    #[cfg(not(unix))]
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

impl FixturePermissions {
    fn read(path: &Path) -> Self {
        let permissions = fs::metadata(path).expect("fixture metadata").permissions();
        Self {
            #[cfg(not(unix))]
            readonly: permissions.readonly(),
            #[cfg(unix)]
            mode: permissions.mode(),
        }
    }

    fn apply(&self, path: &Path) {
        #[cfg(unix)]
        let permissions = fs::Permissions::from_mode(self.mode);
        #[cfg(not(unix))]
        let permissions = {
            let mut permissions = fs::metadata(path)
                .expect("fixture metadata before permission restore")
                .permissions();
            permissions.set_readonly(self.readonly);
            permissions
        };
        fs::set_permissions(path, permissions).expect("restore fixture permissions");
    }

    fn make_writable(path: &Path, _is_directory: bool) {
        let mut permissions = fs::metadata(path)
            .expect("fixture metadata before removal")
            .permissions();
        #[cfg(unix)]
        {
            let minimum = if _is_directory { 0o700 } else { 0o600 };
            permissions.set_mode(permissions.mode() | minimum);
        }
        #[cfg(not(unix))]
        {
            #[cfg_attr(
                windows,
                expect(
                    clippy::permissions_set_readonly_false,
                    reason = "On Windows this clears FILE_ATTRIBUTE_READONLY; Unix uses owner-only mode bits above."
                )
            )]
            permissions.set_readonly(false);
        }
        fs::set_permissions(path, permissions).expect("make fixture entry removable");
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum FixtureNode {
    Directory,
    File(Vec<u8>),
    Symlink { target: PathBuf, directory: bool },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureEntry {
    relative_path: PathBuf,
    node: FixtureNode,
    permissions: Option<FixturePermissions>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FixtureTreeSnapshot {
    root_permissions: FixturePermissions,
    entries: Vec<FixtureEntry>,
}

impl FixtureTreeSnapshot {
    fn capture(root: &Path) -> Self {
        fn walk(root: &Path, current: &Path, entries: &mut Vec<FixtureEntry>) {
            let mut children: Vec<_> = fs::read_dir(current)
                .expect("read fixture directory")
                .map(|entry| entry.expect("fixture directory entry").path())
                .collect();
            children.sort();
            for path in children {
                let relative_path = path
                    .strip_prefix(root)
                    .expect("fixture child stays beneath root")
                    .to_path_buf();
                let metadata = fs::symlink_metadata(&path).expect("fixture entry metadata");
                let (node, permissions) = if metadata.file_type().is_symlink() {
                    let directory = fs::metadata(&path)
                        .map(|target| target.is_dir())
                        .unwrap_or(false);
                    (
                        FixtureNode::Symlink {
                            target: fs::read_link(&path).expect("fixture symlink target"),
                            directory,
                        },
                        None,
                    )
                } else if metadata.is_dir() {
                    let permissions = FixturePermissions::read(&path);
                    walk(root, &path, entries);
                    (FixtureNode::Directory, Some(permissions))
                } else if metadata.is_file() {
                    (
                        FixtureNode::File(fs::read(&path).expect("fixture file bytes")),
                        Some(FixturePermissions::read(&path)),
                    )
                } else {
                    panic!("unsupported fixture entry type: {}", path.display());
                };
                entries.push(FixtureEntry {
                    relative_path,
                    node,
                    permissions,
                });
            }
        }

        let mut entries = Vec::new();
        walk(root, root, &mut entries);
        entries.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
        Self {
            root_permissions: FixturePermissions::read(root),
            entries,
        }
    }

    fn restore(&self, root: &Path) {
        fn clear(path: &Path) {
            let children: Vec<_> = fs::read_dir(path)
                .expect("read fixture directory before restore")
                .map(|entry| entry.expect("fixture directory entry").path())
                .collect();
            for child in children {
                let metadata = fs::symlink_metadata(&child).expect("fixture entry metadata");
                if metadata.file_type().is_symlink() {
                    #[cfg(windows)]
                    {
                        if fs::metadata(&child)
                            .map(|target| target.is_dir())
                            .unwrap_or(false)
                        {
                            fs::remove_dir(&child).expect("remove fixture directory symlink");
                        } else {
                            fs::remove_file(&child).expect("remove fixture file symlink");
                        }
                    }
                    #[cfg(not(windows))]
                    fs::remove_file(&child).expect("remove fixture symlink");
                } else if metadata.is_dir() {
                    FixturePermissions::make_writable(&child, true);
                    clear(&child);
                    fs::remove_dir(&child).expect("remove fixture directory");
                } else {
                    FixturePermissions::make_writable(&child, false);
                    fs::remove_file(&child).expect("remove fixture file");
                }
            }
        }

        FixturePermissions::make_writable(root, true);
        clear(root);

        let mut directories: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| entry.node == FixtureNode::Directory)
            .collect();
        directories.sort_by_key(|entry| entry.relative_path.components().count());
        for entry in &directories {
            fs::create_dir(root.join(&entry.relative_path)).expect("restore fixture directory");
        }

        for entry in &self.entries {
            let path = root.join(&entry.relative_path);
            match &entry.node {
                FixtureNode::Directory => {}
                FixtureNode::File(bytes) => {
                    fs::write(&path, bytes).expect("restore fixture file bytes");
                    entry
                        .permissions
                        .as_ref()
                        .expect("file permissions")
                        .apply(&path);
                }
                FixtureNode::Symlink { target, directory } => {
                    #[cfg(unix)]
                    std::os::unix::fs::symlink(target, &path).expect("restore fixture symlink");
                    #[cfg(windows)]
                    if *directory {
                        std::os::windows::fs::symlink_dir(target, &path)
                            .expect("restore fixture directory symlink");
                    } else {
                        std::os::windows::fs::symlink_file(target, &path)
                            .expect("restore fixture file symlink");
                    }
                    #[cfg(not(any(unix, windows)))]
                    panic!(
                        "symlink restoration is unsupported on this platform: {}",
                        path.display()
                    );
                    let _ = directory;
                }
            }
        }

        directories
            .sort_by_key(|entry| std::cmp::Reverse(entry.relative_path.components().count()));
        for entry in directories {
            entry
                .permissions
                .as_ref()
                .expect("directory permissions")
                .apply(&root.join(&entry.relative_path));
        }
        self.root_permissions.apply(root);
        assert_eq!(
            Self::capture(root),
            *self,
            "fixture restoration must reproduce all file bytes and permissions"
        );
    }
}

#[test]
fn fixture_tree_snapshot_restores_bytes_and_permissions() {
    let directory = tempfile::tempdir().expect("fixture snapshot test directory");
    let root = directory.path();
    fs::create_dir_all(root.join("nested")).expect("nested directory");
    fs::write(root.join("nested/file"), b"original bytes").expect("fixture file");
    #[cfg(unix)]
    fs::set_permissions(root.join("nested/file"), fs::Permissions::from_mode(0o640))
        .expect("set fixture mode");
    let baseline = FixtureTreeSnapshot::capture(root);
    fs::write(root.join("nested/file"), b"mutated bytes").expect("mutate fixture file");
    fs::write(root.join("new-file"), b"remove me").expect("new fixture file");
    baseline.restore(root);
    assert_eq!(FixtureTreeSnapshot::capture(root), baseline);
}

#[cfg(windows)]
#[test]
fn fixture_tree_snapshot_restores_windows_readonly_file() {
    let directory = tempfile::tempdir().expect("fixture snapshot test directory");
    let root = directory.path();
    let file = root.join("readonly-file");
    fs::write(&file, b"original bytes").expect("fixture file");
    let mut permissions = fs::metadata(&file).expect("fixture metadata").permissions();
    permissions.set_readonly(true);
    fs::set_permissions(&file, permissions).expect("make fixture file readonly");

    let baseline = FixtureTreeSnapshot::capture(root);
    fs::write(root.join("extra-file"), b"remove me").expect("extra fixture file");
    baseline.restore(root);

    assert_eq!(
        fs::read(&file).expect("restored fixture bytes"),
        b"original bytes"
    );
    assert!(
        fs::metadata(&file)
            .expect("restored fixture metadata")
            .permissions()
            .readonly()
    );
    assert_eq!(FixtureTreeSnapshot::capture(root), baseline);
}

fn repository() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("temporary repository");
    git(root.path(), &["init", "-q", "--initial-branch=main"]);
    git(root.path(), &["config", "core.autocrlf", "false"]);
    git(
        root.path(),
        &["config", "user.email", "acceptance@example.invalid"],
    );
    git(root.path(), &["config", "user.name", "Acceptance"]);
    fs::write(root.path().join("README.md"), "amendment acceptance\n").expect("README");
    git(root.path(), &["add", "."]);
    git(root.path(), &["commit", "-qm", "initial"]);
    attach(root.path()).expect("attach repository");
    start_work_item_with_options(
        root.path(),
        WORK_ITEM_ID,
        "exercise typed Contract amendment adapters",
        "record a plan change identically through CLI and MCP",
        &["README.md".into()],
        &WorkItemStartOptions {
            authority: "authorized".into(),
            acceptance_criteria: vec!["both adapters return one immutable receipt".into()],
            ..WorkItemStartOptions::default()
        },
    )
    .expect("start Work Item");
    root
}

fn candidate_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ai-cockpit"))
}

fn run_cli(args: &[&str]) -> Output {
    Command::new(candidate_binary())
        .args(args)
        .output()
        .expect("candidate CLI process")
}

fn assert_success(output: &Output, label: &str) {
    assert!(
        output.status.success(),
        "{label} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn candidate_runtime() -> RuntimeContext {
    // Cargo supplies one immutable candidate executable for this test process.
    // Hash its actual bytes once; every fixture still gets its own context clone.
    // Rehashing the large debug binary for each control/MCP call can exhaust the
    // native 600-second test deadline without adding an identity observation.
    static RUNTIME: OnceLock<RuntimeContext> = OnceLock::new();
    RUNTIME
        .get_or_init(|| RuntimeContext {
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            protocol_version: PROTOCOL_VERSION,
            runtime_digest: Digest::sha256_bytes(
                &fs::read(candidate_binary()).expect("read candidate executable"),
            ),
        })
        .clone()
}

#[test]
fn complete_initial_scenarios_are_saved_before_preflight() {
    let root = repository();
    let scenarios = json!([{
        "scenario": "initial-input", "required": true, "status": "unverified",
        "evidence": [], "expected": "the declared input is saved before preflight",
        "verificationPlan": "git diff --check", "reason": null, "expectedResult": null,
        "expectedOutcome": null, "description": null
    }]);
    let input = tempfile::NamedTempFile::new().expect("input file");
    fs::write(input.path(), serde_json::to_vec(&scenarios).expect("JSON")).expect("input");
    let output = run_cli(&[
        "start",
        "--repo",
        root.path().to_str().expect("path"),
        "--id",
        "WI-CLI-COMPLETE-PLAN",
        "--intent",
        "complete initial plan",
        "--goal",
        "avoid a follow-up scenario amendment",
        "--scope",
        "**",
        "--authority",
        "authorized",
        "--acceptance",
        "initial plan is intact",
        "--verification",
        "git diff --check",
        "--scenario-coverage",
        input.path().to_str().expect("input path"),
        "--prepare",
    ]);
    assert_success(&output, "typed complete start");
    let active = root.path().join(".ai/work-items/active");
    let contract: Value = serde_json::from_slice(
        &fs::read(active.join("WI-CLI-COMPLETE-PLAN.contract.json")).expect("Contract"),
    )
    .expect("Contract JSON");
    assert_eq!(contract["scenarioCoverage"], scenarios);
    let summary: Value = serde_json::from_slice(
        &fs::read(active.join("WI-CLI-COMPLETE-PLAN.summary.json")).expect("Summary"),
    )
    .expect("Summary JSON");
    assert_eq!(summary["scenarioCoverage"], scenarios);
    assert!(summary["preflightContractDigest"].is_string());
    assert!(
        !root
            .path()
            .join(".ai/evidence/WI-CLI-COMPLETE-PLAN.contract-amendments")
            .exists()
    );

    let result = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_start",
        json!({
            "workItemId": "WI-MCP-COMPLETE-PLAN", "intent": "complete initial plan",
            "goal": "avoid a follow-up scenario amendment", "scope": ["**"],
            "authority": "authorized", "acceptanceCriteria": ["initial plan is intact"],
            "verification": ["git diff --check"], "scenarioCoverage": scenarios
        }),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
    let mcp_contract: Value = serde_json::from_slice(
        &fs::read(active.join("WI-MCP-COMPLETE-PLAN.contract.json")).expect("MCP Contract"),
    )
    .expect("MCP Contract JSON");
    assert_eq!(
        mcp_contract["scenarioCoverage"],
        contract["scenarioCoverage"]
    );
}

fn typed_request(root: &Path) -> ContractAmendmentRequest {
    let contract_path = root
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value =
        serde_json::from_slice(&fs::read(contract_path).expect("Contract")).expect("Contract JSON");
    ContractAmendmentRequest {
        development_review: None,
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "plan-change-001".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "The implementation surfaced a more accurate acceptance plan.".into(),
        changes: vec![ContractAmendmentChange {
            path: "/goal".into(),
            operation: ContractAmendmentOperation::Set,
            value: Some(json!(
                "record and verify the implementation-plan adjustment"
            )),
        }],
    }
}

fn confirm_current_fixture_preflight(root: &Path) {
    let active = root.join(".ai/work-items/active");
    let contract: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{WORK_ITEM_ID}.contract.json"))).expect("Contract"),
    )
    .expect("Contract JSON");
    let summary: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{WORK_ITEM_ID}.summary.json"))).expect("Summary"),
    )
    .expect("Summary JSON");
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"decisionEvidence": {
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root),
            "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-19T00:00:00Z",
            "recordedBy": "human:isolated-test-fixture",
            "reason": "synthetic adapter test review for the exact current Contract"
        }}),
        &candidate_runtime(),
    )
    .expect("record exact synthetic Contract review");
}

const RETAINED_FIXTURE_CHECKS: [&str; 4] = [
    "cargo test --locked --offline --test material",
    "cargo fmt --all -- --check",
    "cargo test --locked --offline",
    "cargo clippy --locked --offline --all-targets -- -D warnings",
];

fn retained_cli(root: &Path, args: &[&str]) -> Output {
    retained_command(root, args)
        .output()
        .expect("retained-history fixture CLI")
}

fn retained_command(root: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(candidate_binary());
    command
        .args(args)
        .arg("--repo")
        .arg(root)
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join(".ai/fixture-target"))
        .env("CARGO_BUILD_JOBS", "1")
        .env("CARGO_INCREMENTAL", "0")
        .env("CARGO_NET_OFFLINE", "true");
    command
}

fn retained_document(root: &Path, suffix: &str) -> Value {
    serde_json::from_slice(
        &fs::read(root.join(format!(
            ".ai/work-items/active/{WORK_ITEM_ID}.{suffix}.json"
        )))
        .expect("native fixture document"),
    )
    .expect("native fixture JSON")
}

fn retained_amend(root: &Path, change_id: &str, changes: Value) {
    let contract = retained_document(root, "contract");
    let result = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": {
            "schemaVersion": 1, "changeId": change_id,
            "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason": "Isolated equivalent historical fixture; no real consumer authority.",
            "changes": changes
        }}),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
}

fn retained_material_input(root: &Path) -> Value {
    let output = retained_cli(
        root,
        &["work-item", "material-review", "plan", "--id", WORK_ITEM_ID],
    );
    assert_success(&output, "retained-history material plan");
    let request: Value = serde_json::from_slice(&output.stdout).expect("material request");
    assert_eq!(request["findingCodes"], json!([]), "{request}");
    assert_eq!(request["reviewEnabled"], true);
    json!({
        "schemaVersion": 1, "decision": "accept_permitted_unknowns",
        "requestDigest": request["requestDigest"],
        "reviewerActor": "agent:fixture-reviewer",
        "authoritySource": "test-fixture:bounded-review", "assurance": "self_declared",
        "evidenceRefs": [{"path": "tests/material.rs", "digest":
            Digest::sha256_bytes(&fs::read(root.join("tests/material.rs")).expect("fixture source"))}],
        "rationale": "Read the exact isolated fixture source and bounded syntax Unknown.",
        "residualRisk": "The bounded scanner is incomplete; synthetic fixture scope only."
    })
}

fn retained_record_material(root: &Path, input: &Value) -> Output {
    let file = tempfile::NamedTempFile::new().expect("material decision input");
    fs::write(file.path(), serde_json::to_vec(input).expect("input JSON")).expect("material input");
    retained_cli(
        root,
        &[
            "work-item",
            "material-review",
            "record",
            "--id",
            WORK_ITEM_ID,
            "--input",
            file.path().to_str().expect("input path"),
        ],
    )
}

struct RetainedCanonicalFixture {
    directory: tempfile::TempDir,
    old_evidence: Vec<u8>,
    old_outcome: Vec<u8>,
}

fn retained_canonical_amendment_fixture() -> RetainedCanonicalFixture {
    retained_canonical_amendment_fixture_with_source_change(false)
}

fn retained_canonical_amendment_fixture_with_source_change(
    change_source: bool,
) -> RetainedCanonicalFixture {
    retained_canonical_fixture_with_checks(change_source, RETAINED_FIXTURE_CHECKS)
}

fn retained_canonical_fixture_with_checks(
    change_source: bool,
    checks: [&str; 4],
) -> RetainedCanonicalFixture {
    retained_canonical_fixture_with_producer(change_source, checks, false)
}

fn retained_canonical_fixture_with_producer(
    change_source: bool,
    checks: [&str; 4],
    legacy_descriptor_model: bool,
) -> RetainedCanonicalFixture {
    retained_canonical_fixture_with_fault_budget(
        change_source,
        checks,
        legacy_descriptor_model,
        false,
    )
}

fn retained_canonical_fixture_with_fault_budget(
    change_source: bool,
    checks: [&str; 4],
    legacy_descriptor_model: bool,
    fault_budget: bool,
) -> RetainedCanonicalFixture {
    let directory = tempfile::tempdir().expect("retained fixture");
    let root: &::std::path::Path = directory.path();
    git(root, &["init", "-q", "--initial-branch=main"]);
    git(root, &["config", "user.name", "Retained Fixture"]);
    git(root, &["config", "user.email", "fixture@example.invalid"]);
    fs::write(
        root.join("README.md"),
        "Equivalent lifecycle fixture, not Sentinel.\n",
    )
    .expect("README");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"retained_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("fixture manifest");
    fs::create_dir(root.join("src")).expect("source directory");
    fs::write(
        root.join("src/lib.rs"),
        "pub fn expected_date() -> &'static str {\n    \"2026-10-14\"\n}\n",
    )
    .expect("fixture library");
    let lock = Command::new("cargo")
        .args(["generate-lockfile", "--offline"])
        .current_dir(root)
        .output()
        .expect("fixture lockfile");
    assert_success(
        &lock,
        "generate actual fixture Cargo.lock before source inventory",
    );
    git(root, &["add", "."]);
    git(root, &["commit", "-qm", "fixture source baseline"]);
    assert_success(&retained_cli(root, &["attach"]), "attach retained fixture");
    if fault_budget {
        // Isolated fixture policy only, established before any bound input.
        // The real Work Item and the shared Runtime retain their own policy.
        fs::write(
            root.join(".ai/policy.json"),
            serde_json::to_vec_pretty(&::serde_json::json!({
                "schemaVersion": 1,
                "project": {
                    "policyId": "isolated-fault-budget", "layer": "project",
                    "rules": [{
                        "operation": "modify_source",
                        "approvalMode": "no_human_approval_for_low_risk",
                        "requiredEvidence": [],
                        "verificationRequirement": {
                            "schemaVersion": 1, "requiredTier": "T0",
                            "requiredAssurance": "self_declared",
                            "policyRefs": ["isolated-fault-budget"],
                            "stageRefs": ["task"], "gateRefs": [],
                            "reason": "Bound one isolated negative command", "maxTimeoutSeconds": 1
                        }
                    }]
                }
            }))
            .expect("isolated policy JSON"),
        )
        .expect("isolated finite budget before all fixture bindings");
    }
    if legacy_descriptor_model {
        for command in RETAINED_FIXTURE_CHECKS {
            let mut words = command.split_whitespace();
            let program = words.next().expect("profile program");
            let args = words.map(str::to_owned).collect::<Vec<_>>();
            let profile = cockpit_repository::confirm_profile_update(root, program, &args)
                .expect("confirm only the isolated fixture command profile");
            assert_eq!(profile.state, "calibrated");
        }
    }
    let mut start = Vec::from([
        "start",
        "--id",
        WORK_ITEM_ID,
        "--intent",
        "Exercise preserved canonical verification recovery",
        "--goal",
        "Run all four checks through isolated fixture finish and archive",
        "--risk",
        "normal",
        "--authority",
        "authorized",
        "--out-of-scope",
        "No real consumer or human authorization",
        "--acceptance",
        "All four checks execute and history remains intact",
        "--required-evidence",
        "verification_receipt",
    ]);
    for path in [
        "README.md",
        "Cargo.toml",
        "Cargo.lock",
        "src/lib.rs",
        "tests/material.rs",
    ] {
        start.extend(["--scope", path]);
    }
    for command in checks {
        start.extend(["--verification", command]);
    }
    assert_success(&retained_cli(root, &start), "start retained fixture");
    let mut scenario = ::serde_json::json!({
        "scenario": "old fixture four-check cycle", "required": true,
        "status": "unverified", "evidence": [],
        "expected": "The old cycle executes every declared fixture check.",
        "verificationPlan": "Native verification of all four declared fixture commands."
    });
    retained_amend(
        root,
        "retained-profile-before-checkpoint",
        json!([
            {"path": "/governanceProfile", "operation": "set", "value": {"materialInspectionReview": {
                "schemaVersion": 1, "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown", "assurance": "self_declared",
                "reviewerActor": "agent:fixture-reviewer", "authoritySource": "test-fixture:bounded-review",
                "acceptResidualRisk": true
            }}},
            {"path": "/scenarioCoverage", "operation": "add", "value": scenario}
        ]),
    );
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "initial preflight",
    );
    confirm_current_fixture_preflight(root);
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"scenarioCoverage": [scenario]}),
        &candidate_runtime(),
    )
    .expect("planned fixture scenario");
    assert_success(
        &retained_cli(root, &["checkpoint", "--id", WORK_ITEM_ID]),
        "fixture checkpoint",
    );
    fs::create_dir(root.join("tests")).expect("tests directory");
    fs::write(root.join("tests/material.rs"), "fn material() -> String {\n    let mut label = String::from(\"token\");\n    label.push_str(\"ization\");\n    label\n}\n\n#[test]\nfn composed_label_matches() {\n    assert!(\n        std::env::var_os(\"RETAINED_FIXTURE_FAIL\").is_none(),\n        \"requested isolated fixture failure\"\n    );\n    assert_eq!(material(), \"tokenization\");\n    assert_eq!(retained_fixture::expected_date(), \"2026-10-14\");\n}\n")
        .expect("reviewable committed test source");
    git(root, &["add", "-A"]);
    git(
        root,
        &["commit", "-qm", "commit reviewable fixture material"],
    );
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "old-cycle preflight",
    );
    confirm_current_fixture_preflight(root);
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "old-cycle material receipt",
    );
    if legacy_descriptor_model {
        // The child uses actual repository execution/writer APIs under a
        // declared synthetic producer. It is not the unavailable rc.2 binary
        // and cannot authenticate the historical consumer's executable.
        let output = Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "legacy_descriptor_fixture_child",
                "--test-threads=1",
            ])
            .env("WI1102_LEGACY_FIXTURE_ROOT", root)
            .env("CARGO_TARGET_DIR", root.join(".ai/fixture-target"))
            .env("CARGO_BUILD_JOBS", "1")
            .env("CARGO_INCREMENTAL", "0")
            .env("CARGO_NET_OFFLINE", "true")
            .output()
            .expect("isolated synthetic producer child");
        assert!(
            output.status.success(),
            "four actual synthetic legacy-producer checks failed: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    } else {
        assert_success(
            &retained_cli(
                root,
                &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
            ),
            "old four real fixture checks",
        );
    }
    let old_evidence =
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical exists before blocked Outcome");
    let envelope: Value = serde_json::from_slice(&old_evidence).expect("typed old envelope");
    assert_eq!(envelope["passed"], true);
    assert_eq!(envelope["receipt"]["nodesExecuted"], 4);
    assert_eq!(envelope["receipt"]["nodesReused"], 0);
    assert_eq!(envelope["receipt"]["processesSpawned"], 4);
    let finish = retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]);
    assert!(
        !finish.status.success(),
        "unverified scenario must create a real failed finish"
    );
    let old_outcome =
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.outcome.json")))
            .expect("native blocked Outcome");
    assert_eq!(
        serde_json::from_slice::<Value>(&old_outcome).expect("Outcome")["state"],
        "blocked"
    );
    fs::write(
        root.join(".ai/evidence/fixture-preserved-old-cycle.json"),
        &old_evidence,
    )
    .expect("retain actual old receipt bytes outside current pointer");
    fs::write(
        root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"),
        &old_outcome,
    )
    .expect("retain actual blocked Outcome bytes");
    scenario["status"] = json!("verified");
    scenario["evidence"] = json!([".ai/evidence/fixture-preserved-old-cycle.json"]);
    record_work_item_governance_controls_with_runtime(
        root,
        WORK_ITEM_ID,
        &json!({"scenarioCoverage": [scenario]}),
        &candidate_runtime(),
    )
    .expect("record actual old-cycle scenario evidence");
    if change_source {
        fs::write(
            root.join("README.md"),
            "A committed source change requires fresh verification.\n",
        )
        .expect("source change before the current amendment/review");
        git(root, &["add", "README.md"]);
        git(
            root,
            &["commit", "-qm", "current source after the old verification"],
        );
    }
    retained_amend(
        root,
        "sensitive-after-retained-success-and-failed-finish",
        json!([
            {"path": "/acceptanceCriteria", "operation": "add", "value": "Reverify the amended plan before fixture finish and archive"}
        ]),
    );
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "amended preflight",
    );
    confirm_current_fixture_preflight(root);
    RetainedCanonicalFixture {
        directory,
        old_evidence,
        old_outcome,
    }
}

#[test]
fn legacy_descriptor_fixture_child() {
    use cockpit_repository::{
        RepositoryVerificationPolicy, RepositoryVerificationRequest, persist_verification_attempt,
        record_verification_with_runtime, resolve_verification_route, run_repository_verification,
    };
    use cockpit_verification::{
        VerificationCommand, VerificationPlanReceipt, VerificationReusePolicy,
    };
    if let Ok(mode) = std::env::var("WI1102_FAULT_MODE") {
        let marker = std::env::var_os("WI1102_FAULT_MARKER").expect("owned fixture marker");
        fs::write(marker, b"controlled fault command actually started\n").expect("started marker");
        match mode.as_str() {
            "timeout" => std::thread::sleep(std::time::Duration::from_secs(5)),
            #[cfg(unix)]
            "signal" => {
                Command::new("kill")
                    .args(["-TERM", &std::process::id().to_string()])
                    .status()
                    .expect("terminate only this controlled verification child");
            }
            _ => panic!("unsupported controlled fault mode"),
        }
        panic!("controlled negative command must never report success");
    }
    let Some(root) = std::env::var_os("WI1102_LEGACY_FIXTURE_ROOT") else {
        // Also run as an ordinary acceptance test. No skipped or empty pass:
        // its isolated child executes all four commands and this branch
        // verifies the complete synthetic producer/descriptor model.
        let fixture =
            retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
        let envelope: Value = serde_json::from_slice(&fixture.old_evidence).expect("old model");
        assert_eq!(envelope["runtimeVersion"], "1.0.1-rc.2");
        assert_eq!(envelope["passed"], true);
        assert_eq!(envelope["receipt"]["nodesExecuted"], 4);
        assert_eq!(envelope["receipt"]["nodesReused"], 0);
        assert_eq!(envelope["receipt"]["processesSpawned"], 4);
        let directory = ::std::path::Path::join(fixture.directory.path(), ".ai/evidence");
        let prefix = format!("{WORK_ITEM_ID}.verification-attempt.");
        let attempts = fs::read_dir(directory)
            .expect("native attempts")
            .filter_map(|entry| {
                let path = entry.expect("entry").path();
                path.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".json"))
                    .then(|| {
                        serde_json::from_slice::<Value>(&fs::read(path).expect("attempt bytes"))
                            .expect("native JSON")
                    })
            })
            .filter(|v| v["state"] == "execution_completed")
            .collect::<Vec<_>>();
        assert_eq!(attempts.len(), 1);
        assert_eq!(attempts[0]["receipt"], envelope["receipt"]);
        for command in attempts[0]["commands"].as_array().expect("commands") {
            let record = envelope["receipt"]["executionRecords"]
                .as_array()
                .expect("records")
                .iter()
                .find(|r| r["nodeId"] == command["nodeId"])
                .expect("same node");
            assert_ne!(command["commandDigest"], record["commandDigest"]);
            assert_eq!(command["timeoutSeconds"], record["timeoutSeconds"]);
        }
        return;
    };
    let root: ::std::path::PathBuf = PathBuf::from(root);
    let old = RuntimeContext {
        protocol_version: PROTOCOL_VERSION,
        runtime_version: "1.0.1-rc.2".into(),
        runtime_digest: "sha256:3342f01ceb50cc08109737d6b8d481ec354a2495cc4590bd2907a125149a5d4f"
            .parse()
            .expect("synthetic old identity"),
    };
    let snapshot = cockpit_git::GitRepository::discover(&root)
        .expect("fixture Git")
        .snapshot()
        .expect("fixture snapshot");
    let stage = cockpit_protocol::VerificationStage::Task;
    let route = resolve_verification_route(&root, WORK_ITEM_ID, stage, "local", &snapshot)
        .expect("actual policy route");
    let requests = RETAINED_FIXTURE_CHECKS
        .iter()
        .enumerate()
        .map(|(i, command)| {
            let mut words = command.split_whitespace();
            RepositoryVerificationRequest {
                node_id: format!("project-command-{i}"),
                program: words.next().expect("program").into(),
                args: words.map(str::to_owned).collect(),
                scope: vec!["**".into()],
                stage: "task".into(),
                runner: "local".into(),
                runtime_digest: old.runtime_digest.to_string(),
                base_commit: route.base_revision.clone(),
                workers: 1,
                work_item_id: Some(WORK_ITEM_ID.into()),
                timeout_seconds: None,
                policy: RepositoryVerificationPolicy::ProfileAuthorized,
            }
        })
        .collect::<Vec<_>>();
    let mut receipts = requests
        .iter()
        .map(|request| {
            let run = run_repository_verification(&root, request)
                .expect("real old-context fixture execution");
            assert!(run.receipt.passed);
            assert_eq!(run.receipt.processes_spawned, 1);
            assert_eq!(run.receipt.nodes_reused, 0);
            assert_eq!(run.receipt.receipt_candidates.len(), 1);
            assert!(root.join(".ai/evidence/reuse/index.json").is_file());
            let cached = run_repository_verification(&root, request)
                .expect("same old-context profile cache must actually be reusable");
            assert!(cached.receipt.passed);
            assert_eq!(cached.receipt.processes_spawned, 0);
            assert_eq!(cached.receipt.nodes_reused, 1);
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("fixture clock")
                .as_secs() as i64;
            let old_plan = cockpit_repository::plan_repository_verification_action(
                &root, request, &snapshot, now,
            )
            .expect("actual old-runtime cache lookup");
            assert_eq!(old_plan["action"], "reuse");
            let mut current_request = request.clone();
            current_request.runtime_digest = candidate_runtime().runtime_digest.to_string();
            let current_plan = cockpit_repository::plan_repository_verification_action(
                &root,
                &current_request,
                &snapshot,
                now,
            )
            .expect("actual current-runtime cache lookup");
            assert_eq!(current_plan["action"], "execute");
            assert_eq!(
                current_plan["bindingMismatches"],
                json!(["environmentDigest"])
            );
            let input = cockpit_repository::VerificationContextInput {
                program: request.program.clone(),
                args: request.args.clone(),
                command_digest: run.receipt.execution_records[0].command_digest.clone(),
                scope: request.scope.clone(),
                stage: request.stage.clone(),
                runner: request.runner.clone(),
                runtime_digest: request.runtime_digest.clone(),
                base_commit: request.base_commit.clone(),
            };
            let context = |input: &cockpit_repository::VerificationContextInput| {
                match cockpit_repository::assess_verification_reuse(&root, &snapshot, input)
                    .expect("actual profile-context assessment")
                {
                    cockpit_repository::VerificationReuseAssessment::Authorized(authorization) => {
                        authorization.context
                    }
                    cockpit_repository::VerificationReuseAssessment::Denied { reason } => {
                        panic!("fixture profile must be authorized: {reason}")
                    }
                }
            };
            let old_context = context(&input);
            let mut current_input = input;
            current_input.runtime_digest = candidate_runtime().runtime_digest.to_string();
            let mut current_context = context(&current_input);
            assert_ne!(
                old_context.environment_digest,
                current_context.environment_digest
            );
            current_context.environment_digest = old_context.environment_digest.clone();
            assert_eq!(
                old_context, current_context,
                "only producer Runtime changed"
            );
            serde_json::to_value(run.receipt).expect("actual receipt JSON")
        })
        .collect::<Vec<_>>();
    let mut receipt = receipts.remove(0);
    for other in receipts {
        for key in ["results", "executionRecords", "receiptCandidates"] {
            receipt[key].as_array_mut().expect("receipt array").extend(
                other[key]
                    .as_array()
                    .expect("actual records")
                    .iter()
                    .cloned(),
            );
        }
        for key in [
            "nodesPlanned",
            "nodesExecuted",
            "nodesReused",
            "rerunStale",
            "rerunUnknown",
            "protectedNodesExecuted",
            "protectedNodesSkipped",
            "processesSpawned",
            "processSpawnFailures",
            "gitCalls",
            "filesRead",
            "filesHashed",
            "elapsedMs",
        ] {
            receipt[key] = json!(
                receipt[key].as_u64().expect("actual counter")
                    + other[key].as_u64().expect("actual counter")
            );
        }
    }
    let mut plan = VerificationPlanReceipt::new(
        stage,
        route.actual_tier,
        route.actual_tier,
        route.actual_assurance,
        vec!["synthetic old-context actual execution".into()],
        Vec::new(),
    )
    .expect("plan");
    plan.work_item_id = Some(WORK_ITEM_ID.into());
    plan.repository_id = Some(repository_id(&root).to_string());
    plan.repository_snapshot_digest = Some(
        cockpit_repository::snapshot_digest(&snapshot)
            .expect("snapshot digest")
            .to_string(),
    );
    plan.base_revision = route.base_revision;
    plan.affected_paths = route.affected_paths;
    plan.dependency_confidence = Some(route.dependency_confidence);
    if let Some(policy) = route.policy_plan {
        plan.required_tier = Some(policy.requirement.required_tier);
        plan.required_assurance = Some(policy.requirement.required_assurance);
        plan.policy_refs = policy.requirement.policy_refs;
    }
    plan.executed_nodes = requests.iter().map(|r| r.node_id.clone()).collect();
    plan.timeout_seconds = Some(cockpit_verification::DEFAULT_EXECUTION_SECONDS);
    receipt["planReceipt"] = json!(plan);
    assert_eq!(receipt["nodesExecuted"], 4);
    assert_eq!(receipt["nodesReused"], 0);
    let recorded = persist_verification_attempt(
        &root,
        WORK_ITEM_ID,
        &requests,
        &snapshot,
        &old,
        "execution_completed",
        None,
        Some(&receipt),
    )
    .expect("actual native attempt writer");
    let mut attempt: Value = serde_json::from_slice(
        &fs::read(root.join(recorded["path"].as_str().expect("native attempt path")))
            .expect("actual native attempt bytes"),
    )
    .expect("actual native attempt JSON");
    record_verification_with_runtime(&root, WORK_ITEM_ID, &receipt, &old, &snapshot)
        .expect("actual typed fixture writer");
    let original_id = attempt["attemptId"]
        .as_str()
        .expect("attempt id")
        .strip_prefix("sha256:")
        .expect("hex")
        .to_owned();
    let original_path = root.join(format!(
        ".ai/evidence/{WORK_ITEM_ID}.verification-attempt.{original_id}.json"
    ));
    fs::create_dir_all(root.join(".ai/fixture-preserved")).expect("preserved synthetic inputs");
    fs::rename(
        original_path,
        root.join(".ai/fixture-preserved/current-writer-before-legacy-model.json"),
    )
    .expect("preserve fixture writer bytes");
    for (i, request) in requests.iter().enumerate() {
        let descriptor = VerificationCommand::new(
            &request.node_id,
            &request.program,
            request.args.clone(),
            VerificationReusePolicy::NeverReuse,
        )
        .with_current_dir(fs::canonicalize(&root).expect("fixture cwd"))
        .with_timeout_seconds(cockpit_verification::DEFAULT_EXECUTION_SECONDS)
        .command_digest();
        assert_ne!(attempt["commands"][i]["commandDigest"], descriptor);
        attempt["commands"][i]["commandDigest"] = json!(descriptor);
    }
    attempt
        .as_object_mut()
        .expect("attempt object")
        .remove("attemptId");
    let id = cockpit_protocol::digest_json(&attempt).expect("synthetic legacy attempt digest");
    attempt["attemptId"] = json!(id);
    fs::write(
        root.join(format!(
            ".ai/evidence/{WORK_ITEM_ID}.verification-attempt.{}.json",
            id.as_str().strip_prefix("sha256:").expect("hex")
        )),
        serde_json::to_vec_pretty(&attempt).expect("synthetic attempt JSON"),
    )
    .expect("isolated legacy-shape fixture only");
}

#[test]
fn legacy_descriptor_gap_cli_mcp_share_material_admission_and_replay_rejection() {
    let fixture = retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
    let root = fixture.directory.path();
    let cli = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(&cli, "CLI legacy admission");
    let cli: Value = serde_json::from_slice(&cli.stdout).expect("CLI status");
    let mcp = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(mcp.pointer("/result/isError"), Some(&json!(false)), "{mcp}");
    assert_eq!(
        cli["safeActions"],
        mcp["result"]["structuredContent"]["safeActions"]
    );
    assert_eq!(
        cli["diagnostics"],
        mcp["result"]["structuredContent"]["diagnostics"]
    );
    let decision = retained_material_input(root);
    let recorded = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": decision}),
    );
    assert_eq!(
        recorded.pointer("/result/isError"),
        Some(&json!(false)),
        "{recorded}"
    );
    let summary = retained_document(root, "summary");
    let replay = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": decision}),
    );
    assert_eq!(
        replay.pointer("/result/isError"),
        Some(&json!(true)),
        "{replay}"
    );
    assert_eq!(retained_document(root, "summary"), summary);
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "CLI actual service after MCP review");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesToExecute"], 4);
    assert_eq!(plan["nodesReused"], 0);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical"),
        fixture.old_evidence
    );
}

#[test]
fn legacy_descriptor_gap_preserves_original_integrity_and_current_guard_rejections() {
    let fixture = retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
    for case in [
        "foreign_repository",
        "foreign_work_item",
        "non_predecessor",
        "receipt_digest",
        "nested_failure",
        "missing_execution_records",
        "failed_execution_exit",
        "plan_result_mismatch",
        "missing_bound_plan",
        "missing_original_attempt",
        "foreign_command_argv_rehashed_attempt",
        "duplicate_attempt_keys",
        "producer_runtime_identity",
        "unsupported_schema",
        "unsupported_protocol",
        "legacy_untyped",
        "digest_only",
        "bad_marker",
        "bad_transition",
        "missing_current_review",
        "bad_journal",
        "dirty_source",
        "finding",
        "stale_request",
        "invalid_retry",
        "malformed_json",
        "duplicate_keys",
        "wrong_command_without_manifest",
    ] {
        assert_retained_history_rejection(&fixture, case);
    }
}

#[test]
fn legacy_descriptor_gap_requires_four_fresh_checks_without_promoting_old_proof() {
    let fixture = retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
    let root = fixture.directory.path();
    let status = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(&status, "legacy identity status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_ne!(status["verification"], "verified");
    assert!(
        status["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|v| v
                .as_str()
                .is_some_and(|s| s.contains("historical_execution_identity_unverifiable"))),
        "{status}"
    );
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "fresh-only exact material review",
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "plan all four current checks");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesPlanned"], 4);
    assert_eq!(plan["nodesToExecute"], 4);
    assert_eq!(plan["nodesReused"], 0);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical"),
        fixture.old_evidence
    );
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "four actual current checks",
    );
    let current: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("new canonical"),
    )
    .expect("envelope");
    assert_eq!(
        current["runtimeDigest"],
        json!(candidate_runtime().runtime_digest)
    );
    assert_eq!(current["receipt"]["nodesExecuted"], 4);
    assert_eq!(current["receipt"]["nodesReused"], 0);
    assert_eq!(current["receipt"]["processesSpawned"], 4);
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("retained original"),
        fixture.old_evidence
    );
}

#[test]
fn source_change_after_material_review_refreshes_preflight_and_executes_four_checks() {
    let fixture = retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
    let root = fixture.directory.path();
    let decision = retained_material_input(root);
    assert_success(
        &retained_record_material(root, &decision),
        "current material review before source change",
    );
    let before = retained_document(root, "summary");
    let contract = retained_document(root, "contract");
    let receipt_path = root.join(
        before["materialReviewReceipt"]["path"]
            .as_str()
            .expect("material receipt path"),
    );
    let old_material = fs::read(&receipt_path).expect("old material receipt");
    let material_path = root.join("tests/material.rs");
    let mut material = fs::read_to_string(&material_path).expect("current test source");
    material.push_str("\n// Source changed after the current material review.\n");
    fs::write(&material_path, material).expect("in-scope source change");
    git(root, &["add", "tests/material.rs"]);
    git(
        root,
        &["commit", "-qm", "source after current material review"],
    );

    let cli = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(&cli, "stale material status");
    let status: Value = serde_json::from_slice(&cli.stdout).expect("status JSON");
    assert_eq!(status["humanDecisionRequired"], false);
    let actions = status["safeActions"].as_array().expect("safe actions");
    for denied in [
        "run_verification",
        "record_material_review_decision",
        "finish",
    ] {
        assert!(!actions.contains(&json!(denied)), "{status}");
    }
    let mcp = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(mcp.pointer("/result/isError"), Some(&json!(false)), "{mcp}");
    assert_eq!(
        status["safeActions"],
        mcp["result"]["structuredContent"]["safeActions"]
    );
    assert_eq!(retained_document(root, "contract"), contract);
    assert_eq!(
        fs::read(&receipt_path).expect("preserved material"),
        old_material
    );
    assert_eq!(retained_document(root, "summary")["checkpointCount"], 1);
    assert_eq!(
        retained_document(root, "summary")["decisionEvidence"],
        before["decisionEvidence"]
    );
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    assert_success(
        &retained_cli(root, &["preflight", "--contract", &relative]),
        "bounded source-refresh preflight",
    );
    assert!(
        !retained_record_material(root, &decision).status.success(),
        "old typed request remains stale"
    );
    assert_eq!(
        fs::read(&receipt_path).expect("original material receipt"),
        old_material
    );
    assert_eq!(retained_document(root, "outcome")["state"], "blocked");
    assert!(
        !retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--plan-only"]
        )
        .status
        .success()
    );
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "new exact current material review",
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "plan four current checks");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesToExecute"], 4);
    assert_eq!(plan["nodesReused"], 0);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical"),
        fixture.old_evidence
    );
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "four actual fresh checks after refresh and review",
    );
    let evidence: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("new canonical"),
    )
    .expect("evidence JSON");
    assert_eq!(evidence["passed"], true);
    assert_eq!(
        evidence["runtimeDigest"],
        json!(candidate_runtime().runtime_digest)
    );
    assert_eq!(evidence["receipt"]["nodesExecuted"], 4);
    assert_eq!(evidence["receipt"]["nodesReused"], 0);
    assert_eq!(evidence["receipt"]["processesSpawned"], 4);
    assert!(
        evidence["receipt"]["executionRecords"]
            .as_array()
            .expect("records")
            .iter()
            .all(|r| r["passed"] == true && r["spawned"] == true && r["exitCode"] == 0)
    );
    assert_eq!(
        retained_document(root, "summary")["decisionEvidence"],
        before["decisionEvidence"]
    );
    assert_eq!(retained_document(root, "contract"), contract);
    assert_eq!(
        fs::read(&receipt_path).expect("old immutable receipt"),
        old_material
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("preserved history"),
        fixture.old_evidence
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"))
            .expect("preserved Outcome"),
        fixture.old_outcome
    );
    assert_success(
        &retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]),
        "finish after fresh proof",
    );
    assert_success(
        &retained_cli(root, &["archive", "--id", WORK_ITEM_ID, "--json"]),
        "archive after finish",
    );
}

#[test]
fn retained_canonical_after_amendment_runs_four_checks_and_finishes_then_archives() {
    // A committed source change invalidates otherwise legitimate attempt reuse.
    // The real consumer also changes producer Runtime; this portable equivalent
    // fixture uses source freshness without fabricating a historical producer.
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    let root = fixture.directory.path();
    let material = retained_record_material(root, &retained_material_input(root));
    assert_success(
        &material,
        "retained canonical must admit exact current material recovery",
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "plan four fresh checks");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesPlanned"], 4);
    assert_eq!(plan["nodesToExecute"], 4);
    assert_eq!(plan["nodesReused"], 0);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical"),
        fixture.old_evidence
    );
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "all four fresh checks",
    );
    let evidence: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("fresh canonical"),
    )
    .expect("fresh envelope");
    assert_eq!(evidence["passed"], true);
    assert_eq!(evidence["receipt"]["nodesExecuted"], 4);
    assert_eq!(evidence["receipt"]["nodesReused"], 0);
    assert_eq!(evidence["receipt"]["processesSpawned"], 4);
    assert_eq!(evidence["receipt"]["maxConcurrentProcesses"], 1);
    assert_eq!(
        evidence["receipt"]["executionRecords"]
            .as_array()
            .expect("execution records")
            .len(),
        4
    );
    assert!(
        evidence["receipt"]["executionRecords"]
            .as_array()
            .expect("records")
            .iter()
            .all(|r| r["spawned"] == true && r["passed"] == true && r["exitCode"] == 0)
    );
    let summary = retained_document(root, "summary");
    assert_eq!(
        summary["preflightState"], "green",
        "fresh verification records current green preflight"
    );
    assert_eq!(
        summary["preflightContractDigest"],
        evidence["contractDigest"]
    );
    assert_eq!(
        summary["preflightRepositorySnapshotDigest"],
        evidence["repositorySnapshotDigest"]
    );
    assert_success(
        &retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]),
        "actual fixture finish",
    );
    assert_success(
        &retained_cli(root, &["archive", "--id", WORK_ITEM_ID, "--json"]),
        "actual fixture archive",
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("retained original receipt"),
        fixture.old_evidence
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"))
            .expect("retained original Outcome"),
        fixture.old_outcome
    );
    assert!(
        root.join(format!(
            ".ai/work-items/archive/{WORK_ITEM_ID}.archive.json"
        ))
        .is_file()
    );
}

#[test]
fn retained_history_with_unchanged_execution_bindings_preserves_valid_reuse() {
    let fixture = retained_canonical_amendment_fixture();
    let root = fixture.directory.path();
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    let recorded = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": retained_material_input(root)}),
    );
    assert_eq!(
        recorded.pointer("/result/isError"),
        Some(&json!(false)),
        "{recorded}"
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert_success(&plan, "valid existing attempt reuse");
    let plan: Value = serde_json::from_slice(&plan.stdout).expect("plan JSON");
    assert_eq!(plan["processesSpawned"], 0);
    assert_eq!(plan["nodesPlanned"], 4);
    assert_eq!(plan["nodesToExecute"], 0);
    assert_eq!(plan["nodesReused"], 4);
    assert_eq!(
        old["repositorySnapshotDigest"],
        retained_document(root, "summary")["preflightRepositorySnapshotDigest"]
    );
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical is still historical"),
        fixture.old_evidence
    );
}

#[test]
fn retained_history_integrity_is_separate_from_current_source_freshness() {
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    let root = fixture.directory.path();
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    let current = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(&current, "current source status");
    let current: Value = serde_json::from_slice(&current.stdout).expect("status");
    assert_ne!(
        old["repositorySnapshotDigest"],
        current["sourceDigests"]["repositorySnapshot"]
    );
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "a separately reviewed current source does not corrupt an intact historical receipt",
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json")).expect("original"),
        fixture.old_evidence
    );
}

#[test]
fn retained_history_real_check_failure_cannot_finish_and_exact_retry_runs_fresh() {
    let fixture = retained_canonical_amendment_fixture_with_source_change(true);
    assert_retained_real_failure_and_retry(fixture);
}

#[test]
fn legacy_descriptor_gap_failure_keeps_old_bytes_and_exact_retry_executes_fresh() {
    let fixture = retained_canonical_fixture_with_producer(false, RETAINED_FIXTURE_CHECKS, true);
    assert_retained_real_failure_and_retry(fixture);
}

#[test]
fn legacy_descriptor_gap_actual_timeout_preserves_failed_attempt_and_exact_retry() {
    assert_retained_fault_attempt_and_retry("timeout");
}

#[cfg(unix)]
#[test]
fn legacy_descriptor_gap_actual_signal_preserves_failed_attempt_and_exact_retry() {
    assert_retained_fault_attempt_and_retry("signal");
}

fn assert_retained_fault_attempt_and_retry(mode: &str) {
    let fixture =
        retained_canonical_fixture_with_fault_budget(false, RETAINED_FIXTURE_CHECKS, true, true);
    let root: &::std::path::Path = fixture.directory.path();
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "current material review before the controlled fault",
    );
    let executable = std::env::current_exe().expect("owned test command");
    let marker = root.join(".ai/fixture-fault-started");
    // This extra negative command is not called the four declared checks.
    // Only their later actual execution can satisfy the acceptance plan.
    let failed = retained_command(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--command",
            executable.to_str().expect("owned command path"),
            "--args=--exact,legacy_descriptor_fixture_child,--test-threads=1",
            "--timeout-seconds",
            "1",
        ],
    )
    .env("WI1102_FAULT_MODE", mode)
    .env("WI1102_FAULT_MARKER", &marker)
    .output()
    .expect("actual bounded fault verification");
    assert!(!failed.status.success());
    let receipt: Value = serde_json::from_slice(&failed.stdout).expect("actual failed output");
    assert_eq!(receipt["passed"], false, "{receipt}");
    assert_eq!(receipt["processesSpawned"], 1, "{receipt}");
    assert_eq!(receipt["nodesReused"], 0);
    assert_eq!(receipt["receiptCandidates"], ::serde_json::json!([]));
    assert!(
        marker.is_file(),
        "the fault command, not merely its plan, ran"
    );
    let record = &receipt["executionRecords"][0];
    assert_eq!(record["timeoutSeconds"], 1);
    assert_eq!(record["timedOut"], mode == "timeout");
    if mode == "signal" {
        assert_eq!(record["terminationSignal"], 15);
    }
    assert_eq!(record["exitCode"], Value::Null);
    let evidence = root.join(".ai/evidence");
    let prefix = format!("{WORK_ITEM_ID}.verification-attempt.");
    let failed_attempts = fs::read_dir(&evidence)
        .expect("native attempts")
        .filter_map(|entry| {
            let path = entry.expect("entry").path();
            if !path
                .file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&prefix) && n.ends_with(".json"))
            {
                return None;
            }
            let bytes = fs::read(&path).expect("attempt bytes");
            let value: Value = serde_json::from_slice(&bytes).expect("native attempt");
            (value["state"] == "execution_failed" && value["receipt"]["passed"] == false)
                .then_some((path, bytes, value))
        })
        .collect::<Vec<_>>();
    assert_eq!(failed_attempts.len(), 1);
    let mut strict_receipt = receipt.clone();
    for projection_field in ["plannedNodes", "diagnosticSummary"] {
        strict_receipt
            .as_object_mut()
            .expect("receipt object")
            .remove(projection_field)
            .expect("documented CLI projection field");
    }
    strict_receipt["workItemId"] = ::serde_json::json!(WORK_ITEM_ID);
    assert_eq!(failed_attempts[0].2["receipt"], strict_receipt);
    assert_eq!(
        failed_attempts[0].2["runtimeDigest"],
        ::serde_json::json!(candidate_runtime().runtime_digest)
    );
    let canonical = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    assert_eq!(
        fs::read(&canonical).expect("old canonical"),
        fixture.old_evidence
    );
    assert!(
        !retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"])
            .status
            .success()
    );
    let contract = retained_document(root, "contract");
    let summary = retained_document(root, "summary");
    let outcome = retained_document(root, "outcome");
    let events = fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.events.jsonl")))
        .expect("actual failure history");
    let runtime = candidate_runtime();
    record_recovery_decision(root, WORK_ITEM_ID, &::serde_json::json!({
        "schemaVersion": 1, "decisionId": "work-item-recovery", "decision": "retry",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root),
        "predecessorWorkItemId": WORK_ITEM_ID,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).expect("Summary digest"),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).expect("Outcome digest"),
        "predecessorEventsDigest": Digest::sha256_bytes(&events),
        "runtimeVersion": runtime.runtime_version, "runtimeDigest": runtime.runtime_digest,
        "actor": "human:isolated-fixture", "authoritySource": "test-fixture:actual-fault",
        "reason": "Retry after the actual controlled timeout or signal; no old proof promotion.",
        "decidedAt": "2026-10-19T00:00:00Z", "resumeCondition": "Run all four declared checks"
    }), &runtime).expect("supported exact retry against actual failure");
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "actual four declared checks after the fault",
    );
    let current: Value = serde_json::from_slice(&fs::read(&canonical).expect("new canonical"))
        .expect("new current envelope");
    assert_eq!(current["passed"], true);
    assert_eq!(
        current["runtimeDigest"],
        ::serde_json::json!(runtime.runtime_digest)
    );
    assert_eq!(current["receipt"]["nodesExecuted"], 4);
    assert_eq!(current["receipt"]["processesSpawned"], 4);
    assert_eq!(current["receipt"]["nodesReused"], 0);
    assert_eq!(
        fs::read(&failed_attempts[0].0).expect("preserved actual failure"),
        failed_attempts[0].1
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("original history"),
        fixture.old_evidence
    );
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-blocked-outcome.json"))
            .expect("original Outcome"),
        fixture.old_outcome
    );
    assert_success(
        &retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"]),
        "finish only after four actual passes",
    );
    assert_success(
        &retained_cli(root, &["archive", "--id", WORK_ITEM_ID, "--json"]),
        "archive only after actual finish",
    );
}

fn assert_retained_real_failure_and_retry(fixture: RetainedCanonicalFixture) {
    let root: &::std::path::Path = fixture.directory.path();
    assert_success(
        &retained_record_material(root, &retained_material_input(root)),
        "current material decision before any retry",
    );
    let failed = retained_command(
        root,
        &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
    )
    .env("RETAINED_FIXTURE_FAIL", "1")
    .output()
    .expect("actual failing fixture verification");
    assert!(
        !failed.status.success(),
        "a real required-check failure must remain failed"
    );
    let failed_result: Value =
        serde_json::from_slice(&failed.stdout).expect("failed execution result");
    assert!(
        failed_result["processesSpawned"].as_u64().unwrap_or(0) > 0,
        "{failed_result}"
    );
    assert_eq!(failed_result["passed"], false);
    assert_eq!(
        fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("old canonical survives failed execution"),
        fixture.old_evidence
    );
    assert!(
        !retained_cli(root, &["finish", "--id", WORK_ITEM_ID, "--json"])
            .status
            .success()
    );
    let contract = retained_document(root, "contract");
    let summary = retained_document(root, "summary");
    let outcome = retained_document(root, "outcome");
    let events = fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.events.jsonl")))
        .expect("actual native failure events");
    let runtime = candidate_runtime();
    record_recovery_decision(root, WORK_ITEM_ID, &::serde_json::json!({
        "schemaVersion": 1, "decisionId": "work-item-recovery", "decision": "retry",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root),
        "predecessorWorkItemId": WORK_ITEM_ID,
        "predecessorContractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "predecessorSummaryDigest": cockpit_protocol::digest_json(&summary).expect("Summary digest"),
        "predecessorOutcomeDigest": cockpit_protocol::digest_json(&outcome).expect("Outcome digest"),
        "predecessorEventsDigest": Digest::sha256_bytes(&events),
        "runtimeVersion": runtime.runtime_version, "runtimeDigest": runtime.runtime_digest,
        "actor": "human:isolated-fixture", "authoritySource": "test-fixture:actual-failed-check",
        "reason": "Retry the actual failed fixture execution after current material review; not a material bypass.",
        "decidedAt": "2026-10-19T00:00:00Z", "resumeCondition": "Run one fresh replacement fixture cycle"
    }), &runtime).expect("exact native retry after actual failure");
    assert_success(
        &retained_cli(
            root,
            &["verify", "--work-item", WORK_ITEM_ID, "--workers", "1"],
        ),
        "fresh exact retry without the fixture failure",
    );
    let current: Value = serde_json::from_slice(
        &fs::read(root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")))
            .expect("new canonical"),
    )
    .expect("new envelope");
    assert_eq!(current["receipt"]["nodesExecuted"], 4);
    assert_eq!(current["receipt"]["nodesReused"], 0);
    assert_eq!(current["passed"], true);
    assert_ne!(
        retained_document(root, "summary")["recoveryRetryPending"],
        true
    );
}

fn copy_retained_fixture(source: &RetainedCanonicalFixture) -> RetainedCanonicalFixture {
    fn copy_directory(source: &Path, target: &Path, relative: &Path) {
        for entry in fs::read_dir(source).expect("fixture directory") {
            let entry = entry.expect("fixture entry");
            let relative = relative.join(entry.file_name());
            if relative == Path::new(".ai/fixture-target") {
                continue;
            }
            let destination = target.join(entry.file_name());
            let kind = entry.file_type().expect("fixture entry type");
            if kind.is_dir() {
                fs::create_dir(&destination).expect("fixture copy directory");
                copy_directory(&entry.path(), &destination, &relative);
            } else {
                assert!(kind.is_file(), "fixture copies must not follow symlinks");
                fs::copy(entry.path(), destination).expect("exact fixture file copy");
            }
        }
    }
    let directory = tempfile::tempdir().expect("isolated negative copy");
    copy_directory(source.directory.path(), directory.path(), Path::new(""));
    RetainedCanonicalFixture {
        directory,
        old_evidence: source.old_evidence.clone(),
        old_outcome: source.old_outcome.clone(),
    }
}

fn assert_retained_history_rejection(baseline: &RetainedCanonicalFixture, case: &str) {
    let fixture = copy_retained_fixture(baseline);
    let root: &::std::path::Path = fixture.directory.path();
    let initial = retained_cli(
        root,
        &["work-item", "status", "--id", WORK_ITEM_ID, "--json"],
    );
    assert_success(
        &initial,
        "read copied fixture admission before negative mutation",
    );
    let initial: Value = serde_json::from_slice(&initial.stdout).expect("initial copied status");
    assert!(
        initial["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("record_material_review_decision")),
        "copy must be eligible before its single negative mutation: {initial}"
    );
    let mut input = retained_material_input(root);
    let path = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let mut envelope: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    match case {
        "foreign_repository" => {
            envelope["repositoryId"] = json!(Digest::sha256_bytes(b"foreign repo"))
        }
        "foreign_work_item" => envelope["workItemId"] = json!("WI-FOREIGN"),
        "non_predecessor" => {
            envelope["contractDigest"] = json!(Digest::sha256_bytes(b"not an ancestor Contract"))
        }
        "receipt_digest" => {
            envelope["receiptDigest"] = json!(Digest::sha256_bytes(b"wrong receipt digest"))
        }
        "nested_failure" => {
            envelope["receipt"]["results"][0]["passed"] = json!(false);
            envelope["receiptDigest"] =
                json!(cockpit_protocol::digest_json(&envelope["receipt"]).expect("digest"));
        }
        "missing_coverage_manifest"
        | "wrong_command_without_manifest"
        | "wrong_command_then_remove_manifest" => {
            let manifest = envelope["receipt"]["planReceipt"].get("coverageManifest");
            if case != "wrong_command_without_manifest" {
                assert!(
                    manifest.is_some(),
                    "deletion must mutate a real native manifest"
                );
            }
            let node = manifest
                .and_then(|m| m["nodeIds"][0].as_str())
                .map(str::to_owned);
            if case != "missing_coverage_manifest" {
                let records = envelope["receipt"]["executionRecords"]
                    .as_array_mut()
                    .expect("records");
                let record = if let Some(node) = node {
                    records
                        .iter_mut()
                        .find(|r| r["nodeId"] == node)
                        .expect("manifest node record")
                } else {
                    &mut records[0]
                };
                record["commandDigest"] = json!(Digest::sha256_bytes(
                    b"different command with valid digest format"
                ));
            }
            if case != "wrong_command_without_manifest" {
                envelope["receipt"]["planReceipt"]
                    .as_object_mut()
                    .expect("plan")
                    .remove("coverageManifest");
            }
            envelope["receiptDigest"] =
                json!(cockpit_protocol::digest_json(&envelope["receipt"]).expect("digest"));
        }
        "missing_execution_records"
        | "failed_execution_exit"
        | "plan_result_mismatch"
        | "missing_bound_plan" => {
            match case {
                "missing_execution_records" => envelope["receipt"]["executionRecords"] = json!([]),
                "failed_execution_exit" => {
                    envelope["receipt"]["executionRecords"][0]["exitCode"] = json!(23)
                }
                "plan_result_mismatch" => {
                    envelope["receipt"]["planReceipt"]["executedNodes"][0] = json!("foreign-node")
                }
                "missing_bound_plan" => {
                    envelope["receipt"]
                        .as_object_mut()
                        .expect("receipt")
                        .remove("planReceipt");
                }
                _ => unreachable!(),
            }
            envelope["receiptDigest"] =
                json!(cockpit_protocol::digest_json(&envelope["receipt"]).expect("digest"));
        }
        "missing_original_attempt"
        | "tampered_original_attempt"
        | "foreign_command_argv_rehashed_attempt"
        | "duplicate_attempt_keys"
        | "symlink_original_attempt" => {
            let prefix = format!("{WORK_ITEM_ID}.verification-attempt.");
            let attempt_path = fs::read_dir(root.join(".ai/evidence"))
                .expect("evidence")
                .map(|entry| entry.expect("entry").path())
                .find(|path| {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .is_some_and(|name| name.starts_with(&prefix) && name.ends_with(".json"))
                        && serde_json::from_slice::<Value>(&fs::read(path).expect("attempt"))
                            .is_ok_and(|value| {
                                value["passed"] == true && value["receipt"] == envelope["receipt"]
                            })
                })
                .expect("original native matching execution attempt");
            let bytes = fs::read(&attempt_path).expect("attempt bytes");
            let preserved = attempt_path.with_extension("json.preserved");
            fs::rename(&attempt_path, &preserved).expect("preserve original in negative copy");
            if matches!(
                case,
                "tampered_original_attempt" | "foreign_command_argv_rehashed_attempt"
            ) {
                let mut attempt: Value = serde_json::from_slice(&bytes).expect("attempt JSON");
                if case == "tampered_original_attempt" {
                    attempt["commands"][0]["commandDigest"] =
                        json!(Digest::sha256_bytes(b"different native command"));
                } else {
                    attempt["commands"][0]["args"][0] = json!("metadata");
                }
                attempt.as_object_mut().expect("object").remove("attemptId");
                let digest = cockpit_protocol::digest_json(&attempt).expect("attempt digest");
                attempt["attemptId"] = json!(digest);
                let name = format!(
                    "{prefix}{}.json",
                    digest.as_str().strip_prefix("sha256:").expect("hex")
                );
                fs::write(
                    root.join(".ai/evidence").join(name),
                    serde_json::to_vec_pretty(&attempt).expect("JSON"),
                )
                .expect("rehash negative sidecar");
            } else if case == "duplicate_attempt_keys" {
                let text = String::from_utf8(bytes).expect("UTF8");
                fs::write(
                    attempt_path,
                    format!(
                        "{{\"passed\":true,{}",
                        text.trim_start().strip_prefix('{').expect("object")
                    ),
                )
                .expect("duplicate sidecar keys");
            } else if case == "symlink_original_attempt" {
                #[cfg(unix)]
                std::os::unix::fs::symlink(preserved, attempt_path)
                    .expect("negative sidecar symlink");
                #[cfg(not(unix))]
                panic!("symlink mutation is Unix-only");
            }
        }
        "producer_runtime_identity" => {
            envelope["runtimeDigest"] = json!(Digest::sha256_bytes(b"wrong producer"))
        }
        "unsupported_schema" => envelope["evidenceSchemaVersion"] = json!(99),
        "unsupported_protocol" => envelope["protocolVersion"] = json!(99),
        "legacy_untyped" => envelope["captureMode"] = json!("legacy_untyped"),
        "digest_only" => {
            envelope["captureMode"] = json!("digest_only");
            envelope
                .as_object_mut()
                .expect("envelope")
                .remove("receipt");
        }
        "bad_marker" | "bad_transition" | "missing_current_review" => {
            let summary_path =
                root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
            let mut summary = retained_document(root, "summary");
            if case == "bad_marker" {
                summary["verificationInvalidatedByContractAmendment"]["contractHash"] =
                    json!(Digest::sha256_bytes(b"wrong marker"));
            } else if case == "bad_transition" {
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("revalidation")["previousContractHash"] =
                    json!(Digest::sha256_bytes(b"wrong transition"));
            } else {
                summary
                    .as_object_mut()
                    .expect("Summary")
                    .remove("decisionEvidence");
            }
            fs::write(
                summary_path,
                serde_json::to_vec_pretty(&summary).expect("Summary JSON"),
            )
            .expect("negative fixture mutation");
        }
        "bad_journal" => {
            let directory = root.join(format!(".ai/evidence/{WORK_ITEM_ID}.contract-amendments"));
            let committed = directory.join("00000002.committed.json");
            let mut value: Value =
                serde_json::from_slice(&fs::read(&committed).expect("committed record"))
                    .expect("record JSON");
            value["receipt"]["journalDigest"] = json!(Digest::sha256_bytes(b"wrong journal"));
            fs::write(
                committed,
                serde_json::to_vec_pretty(&value).expect("record JSON"),
            )
            .expect("negative journal mutation");
        }
        "dirty_source" => {
            fs::write(root.join("tests/material.rs"), "fn changed() {}\n").expect("dirty source")
        }
        "finding" => {
            fs::write(root.join("tests/material.rs"), include_str!("../../../tests/conformance/fixtures/repository-prompt-injection/repository/material.txt"))
                .expect("isolated Finding fixture");
            git(root, &["add", "tests/material.rs"]);
            git(root, &["commit", "-qm", "isolated committed Finding"]);
        }
        "stale_request" => input["requestDigest"] = json!(Digest::sha256_bytes(b"stale request")),
        "invalid_retry" => fs::write(
            root.join(format!(".ai/decisions/{WORK_ITEM_ID}.recovery.json")),
            br#"{"decision":"retry","fixture":"invalid"}"#,
        )
        .expect("invalid retry fixture"),
        "malformed_json" | "duplicate_keys" => {}
        #[cfg(unix)]
        "symlink" => {
            fs::remove_file(&path).expect("isolated negative only");
            std::os::unix::fs::symlink(
                root.join(".ai/evidence/fixture-preserved-old-cycle.json"),
                &path,
            )
            .expect("negative symlink fixture");
        }
        _ => panic!("unknown rejection case {case}"),
    }
    if matches!(case, "malformed_json") {
        fs::write(&path, b"{").expect("malformed envelope");
    } else if case == "duplicate_keys" {
        let text = serde_json::to_string(&envelope).expect("envelope");
        fs::write(&path, format!("{{\"passed\":true,{}", &text[1..]))
            .expect("duplicate-key envelope");
    } else if case != "symlink" {
        fs::write(
            &path,
            serde_json::to_vec_pretty(&envelope).expect("envelope JSON"),
        )
        .expect("negative envelope fixture");
    }
    let summary_before =
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")))
            .expect("Summary before rejection");
    let record = retained_record_material(root, &input);
    assert!(
        !record.status.success(),
        "{case} unexpectedly admitted material: {}",
        String::from_utf8_lossy(&record.stdout)
    );
    assert_eq!(
        fs::read(root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")))
            .expect("Summary after rejection"),
        summary_before,
        "{case}"
    );
    let request_hex = input["requestDigest"]
        .as_str()
        .expect("request digest")
        .strip_prefix("sha256:")
        .expect("digest prefix");
    assert!(
        !root
            .join(format!(
                ".ai/evidence/material-inspection-review/{WORK_ITEM_ID}/{request_hex}.json"
            ))
            .exists(),
        "{case} published a successful current material receipt"
    );
    let plan = retained_cli(
        root,
        &[
            "verify",
            "--work-item",
            WORK_ITEM_ID,
            "--workers",
            "1",
            "--plan-only",
        ],
    );
    assert!(
        !plan.status.success(),
        "{case} unexpectedly admitted verification"
    );
    let output: Value = serde_json::from_slice(&plan.stdout).expect("precondition rejection JSON");
    let processes = output
        .pointer("/attempt/processesSpawned")
        .or_else(|| output.get("processesSpawned"));
    assert_eq!(processes, Some(&json!(0)), "{case}: {output}");
    assert_eq!(
        fs::read(root.join(".ai/evidence/fixture-preserved-old-cycle.json"))
            .expect("retained original"),
        fixture.old_evidence,
        "{case}"
    );
}

#[test]
fn retained_history_recovery_rejects_corruption_foreign_bindings_and_unauthorized_inputs() {
    let fixture = retained_canonical_amendment_fixture();
    for case in [
        "foreign_repository",
        "foreign_work_item",
        "non_predecessor",
        "receipt_digest",
        "nested_failure",
        "missing_execution_records",
        "failed_execution_exit",
        "plan_result_mismatch",
        "missing_bound_plan",
        "missing_original_attempt",
        "tampered_original_attempt",
        "foreign_command_argv_rehashed_attempt",
        "duplicate_attempt_keys",
        "producer_runtime_identity",
        "unsupported_schema",
        "unsupported_protocol",
        "legacy_untyped",
        "digest_only",
        "bad_marker",
        "bad_transition",
        "bad_journal",
        "dirty_source",
        "finding",
        "missing_current_review",
        "stale_request",
        "invalid_retry",
        "malformed_json",
        "duplicate_keys",
    ] {
        assert_retained_history_rejection(&fixture, case);
    }
}

#[cfg(unix)]
#[test]
fn retained_history_recovery_rejects_symlinked_canonical_verification() {
    let fixture = retained_canonical_amendment_fixture();
    assert_retained_history_rejection(&fixture, "symlink");
    assert_retained_history_rejection(&fixture, "symlink_original_attempt");
}

#[test]
fn retained_history_rejects_removing_a_native_manifest_and_rehashing_the_receipt() {
    let mut checks = RETAINED_FIXTURE_CHECKS;
    checks[2] = "cargo test --workspace --locked --offline";
    let fixture = retained_canonical_fixture_with_checks(false, checks);
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    assert!(old["receipt"]["planReceipt"]["coverageManifest"].is_object());
    assert_retained_history_rejection(&fixture, "missing_coverage_manifest");
    assert_retained_history_rejection(&fixture, "wrong_command_then_remove_manifest");
}

#[test]
fn retained_history_rejects_unbound_command_digest_in_native_nonpartitioned_receipt() {
    let fixture = retained_canonical_amendment_fixture();
    let old: Value = serde_json::from_slice(&fixture.old_evidence).expect("old envelope");
    assert!(
        old["receipt"]["planReceipt"]
            .get("coverageManifest")
            .is_none()
    );
    assert_retained_history_rejection(&fixture, "wrong_command_without_manifest");
}

fn assert_blocked_material_recovery_adapters(record_via_mcp: bool) {
    let directory = repository();
    let root: &::std::path::Path = directory.path();
    let repo = root.to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.join(&relative);
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let opt_in = json!({
        "schemaVersion": 1,
        "changeId": "adapter-material-opt-in",
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
        "reason": "synthetic material opt-in before checkpoint",
        "changes": [{"path": "/governanceProfile", "operation": "set", "value": {
            "materialInspectionReview": {
                "schemaVersion": 1,
                "permittedUnknown": "repository_material_inspection_unavailable",
                "permittedCause": "readable_committed_rust_syntax_unknown",
                "assurance": "self_declared",
                "reviewerActor": "agent:fixture-reviewer",
                "authoritySource": "test-fixture:bounded-review",
                "acceptResidualRisk": true
            }
        }}]
    });
    let amended = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": opt_in}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    assert_success(
        &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
        "initial fixture preflight",
    );
    confirm_current_fixture_preflight(root);
    assert_success(
        &run_cli(&["checkpoint", "--repo", repo, "--id", WORK_ITEM_ID]),
        "fixture checkpoint",
    );
    let failed = run_cli(&["finish", "--repo", repo, "--id", WORK_ITEM_ID]);
    assert!(
        !failed.status.success(),
        "fixture must persist a failed Outcome"
    );
    let blocked_path = root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.outcome.json"));
    assert_eq!(
        serde_json::from_slice::<Value>(&fs::read(&blocked_path).expect("Outcome"))
            .expect("Outcome JSON")["state"],
        "blocked"
    );

    fs::create_dir_all(root.join("src")).expect("source directory");
    fs::write(root.join("src/material.rs"),
        "fn material() -> String { let mut s = String::from(\"token\"); s.push_str(\"ization\"); s }\n")
        .expect("reviewable source");
    git(root, &["add", "-A"]);
    git(root, &["commit", "-qm", "commit reviewable material"]);
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let scope_change = json!({
        "schemaVersion": 1,
        "changeId": "adapter-sensitive-scope",
        "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
        "reason": "review the committed Rust source in the amended scope",
        "changes": [{"path": "/scope", "operation": "replace",
            "value": ["README.md", "src/material.rs"]}]
    });
    let amended = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": scope_change}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    assert_success(
        &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
        "current amended preflight",
    );
    let before = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&before, "status before current human review");
    let before: Value = serde_json::from_slice(&before.stdout).expect("status JSON");
    assert!(
        !before["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("record_material_review_decision"))
    );
    confirm_current_fixture_preflight(root);
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "CLI status after exact human review");
    let status: Value = serde_json::from_slice(&status.stdout).expect("CLI status JSON");
    assert_eq!(status["governanceState"], "red");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("record_material_review_decision")),
        "{status}"
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification"))
    );
    let mcp_status = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_status.pointer("/result/isError"),
        Some(&json!(false)),
        "{mcp_status}"
    );
    assert_eq!(
        mcp_status["result"]["structuredContent"]["safeActions"],
        status["safeActions"]
    );

    let cli_plan = run_cli(&[
        "work-item",
        "material-review",
        "plan",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&cli_plan, "CLI material plan");
    let cli_plan: Value = serde_json::from_slice(&cli_plan.stdout).expect("CLI plan JSON");
    let mcp_plan = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_plan",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_plan.pointer("/result/isError"),
        Some(&json!(false)),
        "{mcp_plan}"
    );
    assert_eq!(
        mcp_plan["result"]["structuredContent"]["requestDigest"],
        cli_plan["requestDigest"]
    );
    let input = json!({
        "schemaVersion": 1,
        "decision": "accept_permitted_unknowns",
        "requestDigest": cli_plan["requestDigest"],
        "reviewerActor": "agent:fixture-reviewer",
        "authoritySource": "test-fixture:bounded-review",
        "assurance": "self_declared",
        "evidenceRefs": [{"path": "src/material.rs", "digest": cli_plan["materialManifestDigest"]}],
        "rationale": "The committed fixture has the exact bounded Unknown.",
        "residualRisk": "The bounded syntax scanner is incomplete."
    });
    let mut stale_input = input.clone();
    stale_input["requestDigest"] = json!(Digest::sha256_bytes(b"foreign material request"));
    let summary_path = root.join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let summary_before_stale = fs::read(&summary_path).expect("Summary before stale input");
    let stale_file = tempfile::NamedTempFile::new().expect("stale input file");
    fs::write(
        stale_file.path(),
        serde_json::to_vec(&stale_input).expect("stale input JSON"),
    )
    .expect("stale input bytes");
    let stale_cli = run_cli(&[
        "work-item",
        "material-review",
        "record",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--input",
        stale_file.path().to_str().expect("stale input path"),
    ]);
    assert!(
        !stale_cli.status.success(),
        "foreign material request must fail"
    );
    let stale_mcp = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": stale_input}),
    );
    assert_eq!(
        stale_mcp.pointer("/result/isError"),
        Some(&json!(true)),
        "{stale_mcp}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after denied stale inputs"),
        summary_before_stale
    );
    let input_file = tempfile::NamedTempFile::new().expect("material input file");
    fs::write(
        input_file.path(),
        serde_json::to_vec(&input).expect("material input JSON"),
    )
    .expect("material input bytes");
    if record_via_mcp {
        let recorded = mcp_call(
            root,
            &candidate_runtime(),
            "work_item_material_review_record",
            json!({"workItemId": WORK_ITEM_ID, "decision": input.clone()}),
        );
        assert_eq!(
            recorded.pointer("/result/isError"),
            Some(&json!(false)),
            "{recorded}"
        );
    } else {
        let recorded = run_cli(&[
            "work-item",
            "material-review",
            "record",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input_file.path().to_str().expect("input path"),
        ]);
        assert_success(&recorded, "CLI typed material receipt");
    }
    let after = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId": WORK_ITEM_ID}),
    );
    assert_eq!(
        after.pointer("/result/isError"),
        Some(&json!(false)),
        "{after}"
    );
    let after = &after["result"]["structuredContent"];
    assert_eq!(after["governanceState"], "red");
    assert!(
        after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification")),
        "{after}"
    );
    assert!(
        !after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("finish"))
    );
    let cli_after = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&cli_after, "CLI status after typed material receipt");
    let cli_after: Value = serde_json::from_slice(&cli_after.stdout).expect("CLI status JSON");
    assert_eq!(cli_after["safeActions"], after["safeActions"]);
    let summary_before_replay = fs::read(&summary_path).expect("Summary after material receipt");
    let replay = mcp_call(
        root,
        &candidate_runtime(),
        "work_item_material_review_record",
        json!({"workItemId": WORK_ITEM_ID, "decision": input}),
    );
    assert_eq!(
        replay.pointer("/result/isError"),
        Some(&json!(true)),
        "{replay}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary after denied replay"),
        summary_before_replay
    );
}

#[test]
fn blocked_material_recovery_cli_and_mcp_share_exact_admission_and_receipt() {
    assert_blocked_material_recovery_adapters(false);
}

#[test]
fn blocked_material_recovery_mcp_record_matches_cli_and_rejects_replay() {
    assert_blocked_material_recovery_adapters(true);
}

fn mcp_call(root: &Path, runtime: &RuntimeContext, name: &str, arguments: Value) -> Value {
    cockpit_mcp::handle_request_for_repo(
        &json!({
            "jsonrpc":"2.0",
            "id":1,
            "method":"tools/call",
            "params":{"name":name,"arguments":arguments}
        }),
        root,
        runtime,
    )
}

fn amendment_history_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    let directory = root
        .join(".ai/evidence")
        .join(format!("{WORK_ITEM_ID}.contract-amendments"));
    if !directory.exists() {
        return Vec::new();
    }
    let mut records = fs::read_dir(directory)
        .expect("read amendment history")
        .map(|entry| {
            let path = entry.expect("history entry").path();
            let bytes = fs::read(&path).expect("history bytes");
            (path, bytes)
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.0.cmp(&right.0));
    records
}

#[test]
fn amend_help_exposes_typed_request_and_preserves_legacy_adapter() {
    let output = run_cli(&["work-item", "amend", "--help"]);
    assert_success(&output, "work-item amend --help");
    let help = String::from_utf8_lossy(&output.stdout);
    assert!(help.contains("--request"), "typed request missing: {help}");
    assert!(help.contains("--input"), "legacy input missing: {help}");
    assert!(help.contains("--reason"), "legacy reason missing: {help}");
}

#[test]
fn typed_cli_and_mcp_share_receipt_and_read_history_without_writes() {
    let root = repository();
    let request = typed_request(root.path());
    let request_value = serde_json::to_value(&request).expect("request JSON");
    let request_directory = tempfile::tempdir().expect("request directory");
    let request_path = request_directory.path().join("typed-amendment.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&request_value).expect("serialize request"),
    )
    .expect("write request");

    let cli = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_path.to_str().expect("request path"),
    ]);
    assert_success(&cli, "typed CLI amendment");
    let mut cli_receipt: Value = serde_json::from_slice(&cli.stdout).expect("CLI receipt");
    cli_receipt
        .as_object_mut()
        .expect("CLI receipt object")
        .remove("nextAction");

    let runtime = candidate_runtime();
    let mcp = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request_value}),
    );
    assert_eq!(
        mcp.pointer("/result/isError"),
        Some(&Value::Bool(false)),
        "{mcp}"
    );
    assert_eq!(mcp.pointer("/result/structuredContent"), Some(&cli_receipt));

    let legacy_input_path = request_directory.path().join("legacy-additive.json");
    fs::write(
        &legacy_input_path,
        br#"{"scopeAppend":["src/legacy-adapter.rs"]}"#,
    )
    .expect("legacy additive input");
    let legacy = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--input",
        legacy_input_path.to_str().expect("legacy input path"),
        "--reason",
        "Retain compatibility for existing additive callers.",
    ]);
    assert_success(&legacy, "legacy additive CLI amendment");

    let mut invalid_request = request_value.clone();
    invalid_request["unrecognized"] = json!(true);
    let invalid = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":invalid_request}),
    );
    assert_eq!(invalid.pointer("/result/isError"), Some(&Value::Bool(true)));
    assert!(
        invalid["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unknown field"),
        "{invalid}"
    );

    let stale = ContractAmendmentRequest {
        change_id: "plan-change-stale".into(),
        ..request.clone()
    };
    let stale_value = serde_json::to_value(&stale).expect("stale request JSON");
    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let before_conflict = fs::read(&contract_path).expect("Contract before conflict");
    let conflict = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":stale_value}),
    );
    assert_eq!(
        conflict.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        conflict["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("contract_digest_conflict"),
        "{conflict}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after conflict"),
        before_conflict
    );

    let history_before = amendment_history_bytes(root.path());
    let cli_history = run_cli(&[
        "work-item",
        "amendments",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&cli_history, "read CLI amendment history");
    let cli_history: Value = serde_json::from_slice(&cli_history.stdout).expect("CLI history");
    assert_eq!(cli_history.as_array().map(Vec::len), Some(2));
    let mcp_history = mcp_call(
        root.path(),
        &runtime,
        "work_item_amendments",
        json!({"workItemId":WORK_ITEM_ID}),
    );
    assert_eq!(
        mcp_history.pointer("/result/structuredContent"),
        Some(&cli_history)
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);

    let legacy = RuntimeContext {
        runtime_version: "0.2.113".into(),
        protocol_version: PROTOCOL_VERSION,
        runtime_digest: "sha256:c85632062eb5ef8f8b39f6154b765c9e2543fe6b7ca3f54e107a59c174843686"
            .parse()
            .expect("installed predecessor digest"),
    };
    let unsupported = mcp_call(
        root.path(),
        &legacy,
        "work_item_environment_drift",
        json!({"action":"check","workItemId":WORK_ITEM_ID,"generation":1}),
    );
    assert_eq!(
        unsupported.pointer("/result/isError"),
        Some(&Value::Bool(true))
    );
    assert!(
        unsupported["result"]["content"][0]["text"]
            .as_str()
            .unwrap_or_default()
            .contains("unsupported_runtime_capability"),
        "{unsupported}"
    );
}

#[test]
fn inspect_accepts_null_optional_concurrency_boundary_after_amendment() {
    let root = repository();
    let request = serde_json::to_vec_pretty(&typed_request(root.path()))
        .expect("serialize amendment request");
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(request_file.path(), request).expect("write amendment request");

    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "amend Work Item before inspect");

    let contract_path = root
        .path()
        .join(".ai/work-items/active")
        .join(format!("{WORK_ITEM_ID}.contract.json"));
    let contract: Value = serde_json::from_slice(&fs::read(contract_path).expect("read Contract"))
        .expect("Contract JSON");
    assert_eq!(contract["concurrencyBoundary"], Value::Null);

    let inspected = run_cli(&[
        "work-item",
        "inspect",
        "--repo",
        root.path().to_str().expect("repository path"),
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&inspected, "inspect amended Contract with null boundary");
}

fn assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(retry_pending: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");

    let initial_preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial_preflight, "initial preflight");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before amendment");

    let contract_path = root.path().join(&contract_relative);
    if retry_pending {
        let summary_path = root
            .path()
            .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
        let predecessor_contract: Value =
            serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
                .expect("Contract JSON");
        let predecessor_summary: Value =
            serde_json::from_slice(&fs::read(&summary_path).expect("read Summary"))
                .expect("Summary JSON");
        let runtime = candidate_runtime();
        let retry = json!({
            "schemaVersion": 1,
            "decisionId": "work-item-recovery",
            "decision": "retry",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()).to_string(),
            "predecessorWorkItemId": WORK_ITEM_ID,
            "predecessorContractDigest": cockpit_protocol::digest_json(&predecessor_contract).expect("Contract digest"),
            "predecessorSummaryDigest": cockpit_protocol::digest_json(&predecessor_summary).expect("Summary digest"),
            "runtimeVersion": runtime.runtime_version,
            "runtimeDigest": runtime.runtime_digest,
            "actor": "human:fixture",
            "authoritySource": "test fixture",
            "reason": "retry a stale verification attempt",
            "decidedAt": "2026-10-03T00:00:00Z",
            "resumeCondition": "run replacement verification"
        });
        record_recovery_decision(root.path(), WORK_ITEM_ID, &retry, &runtime)
            .expect("bind retry decision through Runtime");
    }
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("read Contract"))
        .expect("Contract JSON");
    let request = ContractAmendmentRequest {
        development_review: None,
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "sensitive-scope-change".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&contract)
            .expect("Contract digest"),
        reason: "scope expansion requires human policy review before verification".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let request_file = tempfile::NamedTempFile::new().expect("amendment request file");
    fs::write(
        request_file.path(),
        serde_json::to_vec_pretty(&request).expect("serialize amendment request"),
    )
    .expect("write amendment request");
    let amended = run_cli(&[
        "work-item",
        "amend",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--request",
        request_file.path().to_str().expect("request path"),
    ]);
    assert_success(&amended, "record sensitive amendment");

    if retry_pending {
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status before retry amendment review preflight");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert!(
            status["safeActions"]
                .as_array()
                .expect("safe actions")
                .iter()
                .any(|action| action == "run_preflight"),
            "a pending retry and sensitive amendment must admit request-generating preflight: {status}"
        );
    }

    let preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(
        &preflight,
        "Runtime-bound preflight should produce a pending amendment review request",
    );
    let preflight: Value = serde_json::from_slice(&preflight.stdout).expect("preflight JSON");
    let current_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("read amended Contract"))
            .expect("amended Contract JSON");
    let current_contract_digest =
        cockpit_protocol::digest_json(&current_contract).expect("amended Contract digest");
    assert_eq!(preflight["reviewState"], "needs_human_confirmation");
    assert_eq!(
        preflight["humanDecisionRequest"]["decisionId"],
        "contract-preflight-review"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["status"],
        "needs_human_confirmation"
    );
    assert_eq!(
        preflight["humanDecisionRequest"]["recommendedOption"],
        "confirm_review"
    );
    assert_eq!(preflight["actionAdmission"]["workItemId"], WORK_ITEM_ID);
    assert_eq!(
        preflight["actionAdmission"]["repositoryId"],
        current_contract["repositoryId"]
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["contract"],
        current_contract_digest.as_str()
    );
    assert_eq!(
        preflight["actionAdmission"]["sourceDigests"]["repositorySnapshot"],
        preflight["actionAdmission"]["snapshotDigest"]
    );
    assert!(
        preflight["actionAdmission"]["snapshotDigest"]
            .as_str()
            .is_some_and(|digest| digest.starts_with("sha256:")),
        "request should be returned with a current repository snapshot binding"
    );
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "status after refreshed preflight");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(status["humanDecisionRequired"], true);
    assert_eq!(status["humanDecisions"], json!([]));
    if retry_pending {
        let summary: Value = serde_json::from_slice(
            &fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("read Summary"),
        )
        .expect("Summary JSON");
        assert_eq!(summary["recoveryRetryPending"], true);

        let current_receipt = json!({
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": WORK_ITEM_ID,
            "repositoryId": repository_id(root.path()),
            "contractDigest": current_contract_digest,
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-03T00:00:00Z",
            "recordedBy": "human:test-fixture",
            "reason": "fixture only; no human decision is recorded"
        });
        assert_eq!(
            current_receipt["repositorySnapshotDigest"],
            preflight["actionAdmission"]["snapshotDigest"]
        );
        let previous_contract_digest =
            cockpit_protocol::digest_json(&contract).expect("pre-amendment Contract digest");
        for (field, wrong_digest, expected_diagnostic) in [
            (
                "repositoryId",
                Digest::sha256_bytes(b"foreign repository").to_string(),
                "repository identity mismatch",
            ),
            (
                "contractDigest",
                previous_contract_digest.to_string(),
                "Contract digest mismatch",
            ),
            (
                "repositorySnapshotDigest",
                Digest::sha256_bytes(b"stale snapshot").to_string(),
                "snapshot digest mismatch",
            ),
        ] {
            let mut invalid_receipt = current_receipt.clone();
            invalid_receipt[field] = Value::String(wrong_digest);
            let summary_before = fs::read(
                root.path()
                    .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
            )
            .expect("Summary before rejected decision");
            let error = record_work_item_governance_controls_with_runtime(
                root.path(),
                WORK_ITEM_ID,
                &json!({"decisionEvidence": invalid_receipt}),
                &candidate_runtime(),
            )
            .expect_err("foreign or stale review evidence must be rejected");
            assert!(
                error.to_string().contains(expected_diagnostic),
                "{field} should fail for its identity mismatch: {error}"
            );
            assert_eq!(
                fs::read(
                    root.path()
                        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json")),
                )
                .expect("Summary after rejected decision"),
                summary_before,
                "{field} rejection must not record a human decision"
            );
        }
        assert!(
            !root
                .path()
                .join(format!(
                    ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
                ))
                .exists(),
            "invalid review evidence must not create a decision receipt"
        );
    }
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification")
    );

    let marker = root.path().join("verification-command-started");
    let mut verify_args = vec![
        "verify".to_owned(),
        "--repo".to_owned(),
        repository_path.to_owned(),
        "--work-item".to_owned(),
        WORK_ITEM_ID.to_owned(),
        "--command".to_owned(),
    ];
    #[cfg(unix)]
    {
        verify_args.push("sh".into());
        verify_args.push("--args=-c".into());
        verify_args.push(format!("--args=printf started > '{}'", marker.display()));
    }
    #[cfg(windows)]
    {
        verify_args.push("cmd".into());
        verify_args.push("--args=/C".into());
        verify_args.push(format!("--args=echo started > \"{}\"", marker.display()));
    }
    let verify_args = verify_args.iter().map(String::as_str).collect::<Vec<_>>();
    let verify = run_cli(&verify_args);
    assert!(
        !verify.status.success(),
        "unreviewed amendment must reject verify"
    );
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    assert!(
        diagnostic.contains("contract_amendment_policy_review_required"),
        "expected the amendment review gate to reject verification, got: {diagnostic}"
    );
    assert!(
        !marker.exists(),
        "verification command must not start before amendment review"
    );
}

#[test]
fn sensitive_amendment_blocks_cli_verification_before_command_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(false);
}

#[test]
fn retry_pending_amendment_admits_bound_review_request_without_verification_spawn() {
    assert_sensitive_amendment_blocks_cli_verification_before_command_spawn(true);
}

fn assert_mixed_pre_amendment_review_binding_is_rejected(use_mcp: bool) {
    let root = repository();
    let repository_path = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let initial = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        &contract_relative,
    ]);
    assert_success(&initial, "initial preflight before mixed-binding fixture");
    let checkpoint = run_cli(&[
        "checkpoint",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
    ]);
    assert_success(&checkpoint, "checkpoint before mixed-binding fixture");
    let old_summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("old Summary"))
        .expect("old Summary JSON");
    let old_contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("old Contract"))
            .expect("old Contract JSON");
    let runtime = candidate_runtime();
    let request = ContractAmendmentRequest {
        development_review: None,
        schema_version: CONTRACT_AMENDMENT_SCHEMA_VERSION,
        change_id: "mixed-binding-sensitive-amendment".into(),
        expected_contract_digest: cockpit_protocol::digest_json(&old_contract)
            .expect("old Contract digest"),
        reason: "isolated regression: invalidate old preflight before any new review".into(),
        changes: vec![ContractAmendmentChange {
            path: "/scope".into(),
            operation: ContractAmendmentOperation::Replace,
            value: Some(json!(["README.md", "src/**"])),
        }],
    };
    let amended = mcp_call(
        root.path(),
        &runtime,
        "work_item_amend",
        json!({"workItemId": WORK_ITEM_ID, "request": request}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    let new_contract_bytes = fs::read(&contract_path).expect("new Contract");
    let new_contract: Value =
        serde_json::from_slice(&new_contract_bytes).expect("new Contract JSON");
    let summary_before = fs::read(&summary_path).expect("invalidated Summary");
    let invalidated: Value =
        serde_json::from_slice(&summary_before).expect("invalidated Summary JSON");
    assert_eq!(invalidated["preflightState"], "not_run");
    assert_eq!(
        invalidated["preflightContractDigest"],
        old_summary["preflightContractDigest"]
    );
    assert_eq!(
        invalidated["preflightDecisionDigest"],
        old_summary["preflightDecisionDigest"]
    );
    let history_before = amendment_history_bytes(root.path());
    // This input exists only inside this temporary regression fixture.
    // A current Contract cannot turn the retained old preflight fields into
    // a new identity-bound human review before Runtime generates that review.
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1,
        "decisionId": "contract-preflight-review",
        "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID,
        "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&new_contract).expect("new Contract digest"),
        "preflightDecisionDigest": old_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": old_summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z",
        "recordedBy": "human:isolated-automated-test-fixture",
        "reason": "negative automated fixture only; not an actual human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &runtime,
            "work_item_controls",
            json!({"workItemId": WORK_ITEM_ID, "controls": controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(true)),
            "MCP must reject a new Contract mixed with pre-amendment preflight fields: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("write controls input");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repository_path,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("controls path"),
        ]);
        assert!(
            !response.status.success(),
            "CLI must reject a new Contract mixed with pre-amendment preflight fields; stdout={} stderr={}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    assert_eq!(
        fs::read(&summary_path).expect("Summary after rejection"),
        summary_before
    );
    assert_eq!(
        fs::read(&contract_path).expect("Contract after rejection"),
        new_contract_bytes
    );
    assert_eq!(amendment_history_bytes(root.path()), history_before);
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists(),
        "rejected mixed binding must not publish a decision receipt"
    );
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_cli() {
    assert_mixed_pre_amendment_review_binding_is_rejected(false);
}

#[test]
fn mixed_current_contract_and_pre_amendment_preflight_receipt_is_rejected_by_mcp() {
    assert_mixed_pre_amendment_review_binding_is_rejected(true);
}

fn assert_recorded_preflight_review_boundary(
    state: Option<&str>,
    current_contract: bool,
    use_mcp: bool,
    accepted: bool,
) {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&contract_relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "record fixture preflight",
    );
    let contract_bytes = fs::read(&contract_path).expect("Contract bytes");
    let contract: Value = serde_json::from_slice(&contract_bytes).expect("Contract JSON");
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    // Isolated boundary fixtures vary the recorded field under test. The
    // native amendment reproduction above independently covers real history.
    if let Some(state) = state {
        summary["preflightState"] = json!(state);
    } else {
        summary
            .as_object_mut()
            .expect("Summary object")
            .remove("preflightState");
    }
    if !current_contract {
        summary["preflightContractDigest"] = json!(Digest::sha256_bytes(b"older Contract"));
    }
    fs::write(
        &summary_path,
        serde_json::to_vec_pretty(&summary).expect("fixture Summary"),
    )
    .expect("write isolated fixture");
    let before = fs::read(&summary_path).expect("Summary before record");
    let controls = json!({"decisionEvidence": {
        "schemaVersion": 1, "decisionId": "contract-preflight-review", "decision": "confirm_review",
        "workItemId": WORK_ITEM_ID, "repositoryId": repository_id(root.path()),
        "contractDigest": cockpit_protocol::digest_json(&contract).expect("Contract digest"),
        "preflightDecisionDigest": summary["preflightDecisionDigest"],
        "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
        "recordedAt": "2026-10-16T00:00:00Z", "recordedBy": "human:isolated-test-fixture",
        "reason": "automated boundary fixture; no real human approval"
    }});
    if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "work_item_controls",
            json!({"workItemId":WORK_ITEM_ID,"controls":controls}),
        );
        assert_eq!(
            response.pointer("/result/isError"),
            Some(&json!(!accepted)),
            "state={state:?}, current_contract={current_contract}: {response}"
        );
    } else {
        let input = tempfile::NamedTempFile::new().expect("controls input");
        fs::write(
            input.path(),
            serde_json::to_vec(&controls).expect("controls JSON"),
        )
        .expect("controls input bytes");
        let response = run_cli(&[
            "work-item",
            "controls",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--input",
            input.path().to_str().expect("input path"),
        ]);
        assert_eq!(
            response.status.success(),
            accepted,
            "state={state:?}, current_contract={current_contract}: {}{}",
            String::from_utf8_lossy(&response.stdout),
            String::from_utf8_lossy(&response.stderr)
        );
    }
    let receipt_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    assert_eq!(
        fs::read(&contract_path).expect("unchanged Contract"),
        contract_bytes
    );
    if !accepted {
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before
        );
        assert!(
            !receipt_path.exists(),
            "rejection must not publish a receipt"
        );
    } else {
        let receipt = fs::read(&receipt_path).expect("actual fixture receipt");
        fs::write(
            root.path().join("README.md"),
            "changed source requires fresh execution, same Contract review remains valid\n",
        )
        .expect("source change");
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&status, "status after same-Contract source change");
        let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
        assert_eq!(
            status["humanDecisionRequired"], false,
            "same-Contract approval must be retained: {status}"
        );
        assert_eq!(fs::read(&receipt_path).expect("preserved receipt"), receipt);
    }
}

#[test]
fn missing_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(None, true, mcp, false);
    }
}

#[test]
fn unknown_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("unknown"), true, mcp, false);
    }
}

#[test]
fn not_run_recorded_preflight_state_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("not_run"), true, mcp, false);
    }
}

#[test]
fn foreign_recorded_preflight_contract_rejects_review_without_writes() {
    for mcp in [false, true] {
        assert_recorded_preflight_review_boundary(Some("yellow"), false, mcp, false);
    }
}

#[test]
fn valid_recorded_preflight_states_preserve_same_contract_approval() {
    for state in ["green", "yellow", "red"] {
        for mcp in [false, true] {
            assert_recorded_preflight_review_boundary(Some(state), true, mcp, true);
        }
    }
}

fn amended_request_fixture(red: bool) -> tempfile::TempDir {
    let root = repository();
    let repo = root.path().to_str().expect("repository path");
    let contract_relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    if !red {
        let project = root.path().join(".ai/project");
        fs::create_dir_all(&project).expect("project directory");
        fs::write(project.join("documentation-policy.json"), serde_json::to_vec(&json!({
            "schemaVersion":1, "repositoryId":repository_id(root.path()), "defaultProjection":"required",
            "requiredModes":["implementation"], "requiredOperations":[], "preserveExistingRegistrations":true
        })).expect("policy JSON")).expect("documentation-first fixture policy");
    }
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            repo,
            "--contract",
            &contract_relative,
        ]),
        "initial request fixture preflight",
    );
    assert_success(
        &run_cli(&["checkpoint", "--repo", repo, "--id", WORK_ITEM_ID]),
        "request fixture checkpoint",
    );
    let contract: Value =
        serde_json::from_slice(&fs::read(root.path().join(&contract_relative)).expect("Contract"))
            .expect("Contract JSON");
    let changes = vec![json!({"path":"/scope","operation":"add","value":"src/**"})];
    let request = json!({"schemaVersion":1,"changeId":"interlocked-request","expectedContractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),"reason":"isolated request generation fixture","changes":changes});
    let result = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId":WORK_ITEM_ID,"request":request}),
    );
    assert_eq!(
        result.pointer("/result/isError"),
        Some(&json!(false)),
        "{result}"
    );
    if red {
        // An intentionally contradictory retained envelope makes native
        // Outcome red. Request generation must preserve this evidence and
        // its blocker, never reinterpret it as a passed verification.
        fs::write(
            root.path()
                .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json")),
            br#"{"evidenceSchemaVersion":2,"fixture":"contradictory retained verification envelope"}"#,
        )
        .expect("contradictory evidence fixture");
    }
    root
}

fn assert_interlocked_request_generation(red: bool, use_mcp: bool) {
    let root = amended_request_fixture(red);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let history = amendment_history_bytes(root.path());
    let evidence_path = root
        .path()
        .join(format!(".ai/evidence/{WORK_ITEM_ID}.verification.json"));
    let evidence_before = fs::read(&evidence_path).ok();
    let status = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&status, "interlocked status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_preflight")),
        "interlocked request generation must be admitted: {status}"
    );
    if red {
        assert_eq!(status["governanceState"], "red");
    }
    let mut previous_snapshot = None;
    for observation in 0..2 {
        if observation == 1 {
            fs::write(
                root.path().join("README.md"),
                "new current snapshot for repeated request\n",
            )
            .expect("source update");
        }
        let decision = if use_mcp {
            let response = mcp_call(
                root.path(),
                &candidate_runtime(),
                "preflight",
                json!({"contract":relative}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(false)),
                "{response}"
            );
            response["result"]["structuredContent"].clone()
        } else {
            let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
            assert_success(&response, "generate interlocked request");
            serde_json::from_slice(&response.stdout).expect("decision JSON")
        };
        assert_eq!(decision["reviewState"], "needs_human_confirmation");
        assert_eq!(
            decision["humanDecisionRequest"]["decisionId"],
            "contract-preflight-review"
        );
        if red {
            assert_eq!(decision["state"], "red");
            assert_eq!(decision["outcome_state"], "blocked");
            assert!(
                !decision["blockers"]
                    .as_array()
                    .expect("blockers")
                    .is_empty()
            );
        }
        let summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        let contract: Value =
            serde_json::from_slice(&fs::read(root.path().join(&relative)).expect("Contract"))
                .expect("Contract JSON");
        assert_eq!(
            summary["preflightContractDigest"],
            json!(cockpit_protocol::digest_json(&contract).expect("Contract digest"))
        );
        if let Some(previous) = previous_snapshot {
            assert_ne!(
                summary["preflightRepositorySnapshotDigest"], previous,
                "repeat must bind the changed current source snapshot"
            );
        }
        previous_snapshot = Some(summary["preflightRepositorySnapshotDigest"].clone());
        assert!(
            summary.get("decisionEvidence").is_none(),
            "request generation never records confirmation"
        );
        assert_eq!(summary["checkpointCount"], 1);
        assert_eq!(amendment_history_bytes(root.path()), history);
        assert_eq!(fs::read(&evidence_path).ok(), evidence_before);
    }
    let after = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repo,
        "--id",
        WORK_ITEM_ID,
        "--json",
    ]);
    assert_success(&after, "pending request status");
    let after: Value = serde_json::from_slice(&after.stdout).expect("status JSON");
    assert_eq!(after["humanDecisionRequired"], true);
    assert!(
        !after["safeActions"]
            .as_array()
            .expect("safe actions")
            .contains(&json!("run_verification"))
    );
    assert!(
        !root
            .path()
            .join(format!(
                ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
            ))
            .exists()
    );
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(false, false);
}

#[test]
fn documentation_first_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(false, true);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_cli() {
    assert_interlocked_request_generation(true, false);
}

#[test]
fn governance_red_interlocked_request_is_generated_by_mcp() {
    assert_interlocked_request_generation(true, true);
}

#[test]
fn interlocked_request_rejects_invalid_provenance_without_writes() {
    let root = amended_request_fixture(true);
    let protocol_root = root.path().join(".ai");
    let baseline_protocol = FixtureTreeSnapshot::capture(&protocol_root);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let contract_path = root.path().join(&relative);
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    for case in [
        "marker_missing",
        "marker_hash",
        "invalidated_checks",
        "revalidation_snapshot",
        "before_edit_hash",
        "summary_repo",
        "summary_wi",
        "unjournaled_contract",
        "journal_tampered",
        "journal_pending",
        "not_ready",
        "closed",
        "recovered",
        "archived",
    ] {
        // Each case begins from the same complete, byte-preserved protocol
        // state. Rebuild the native fixture once, then restore only its .ai
        // tree between adversarial mutations.
        baseline_protocol.restore(&protocol_root);
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "marker_missing" => {
                summary
                    .as_object_mut()
                    .expect("Summary object")
                    .remove("verificationInvalidatedByContractAmendment");
            }
            "marker_hash" => {
                summary["verificationInvalidatedByContractAmendment"]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign Contract"))
            }
            "invalidated_checks" => {
                summary["verificationInvalidatedByContractAmendment"]["invalidatedRequiredChecks"] =
                    json!(["fabricated"]);
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["invalidatedRequiredChecks"] = json!(["fabricated"]);
            }
            "revalidation_snapshot" => {
                summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .last_mut()
                    .expect("last checkpoint")["repositorySnapshotDigest"] =
                    json!(Digest::sha256_bytes(b"foreign snapshot"))
            }
            "before_edit_hash" => {
                summary["checkpointEvidence"][0]["contractHash"] =
                    json!(Digest::sha256_bytes(b"foreign before-edit"))
            }
            "summary_repo" => {
                summary["repositoryId"] = json!(Digest::sha256_bytes(b"foreign repository"))
            }
            "summary_wi" => summary["workItemId"] = json!("WI-FOREIGN"),
            "unjournaled_contract" => {
                let mut contract: Value =
                    serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
                        .expect("Contract JSON");
                contract["goal"] = json!("unrecorded Contract change");
                fs::write(
                    &contract_path,
                    serde_json::to_vec(&contract).expect("Contract JSON"),
                )
                .expect("isolated Contract tamper");
            }
            "journal_tampered" => {
                let path = root.path().join(format!(
                    ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).expect("journal"))
                    .expect("journal JSON");
                value["receipt"]["reason"] = json!("tampered");
                fs::write(path, serde_json::to_vec(&value).expect("journal JSON"))
                    .expect("isolated journal tamper");
            }
            "journal_pending" => fs::remove_file(root.path().join(format!(
                ".ai/evidence/{WORK_ITEM_ID}.contract-amendments/00000001.committed.json"
            )))
            .expect("isolated pending transaction"),
            phase => summary["state"] = json!(phase),
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("Summary JSON"),
        )
        .expect("isolated projection");
        let before_summary = fs::read(&summary_path).expect("Summary before rejection");
        let before_contract = fs::read(&contract_path).expect("Contract before rejection");
        let before_history = amendment_history_bytes(root.path());
        let before_protocol = FixtureTreeSnapshot::capture(&protocol_root);
        let cli = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        assert!(
            !cli.status.success(),
            "{case} must not admit request generation"
        );
        let mcp = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        assert_eq!(
            mcp.pointer("/result/isError"),
            Some(&json!(true)),
            "{case}: {mcp}"
        );
        assert_eq!(
            fs::read(&summary_path).expect("Summary after rejection"),
            before_summary,
            "{case}"
        );
        assert_eq!(
            fs::read(&contract_path).expect("Contract after rejection"),
            before_contract,
            "{case}"
        );
        assert_eq!(
            amendment_history_bytes(root.path()),
            before_history,
            "{case}"
        );
        assert_eq!(
            FixtureTreeSnapshot::capture(&protocol_root),
            before_protocol,
            "{case}: rejected request generation must preserve the entire protocol tree"
        );
    }
}

#[test]
fn interlocked_request_covers_each_eligible_nonterminal_phase() {
    let root = amended_request_fixture(true);
    let protocol_root = root.path().join(".ai");
    let baseline_protocol = FixtureTreeSnapshot::capture(&protocol_root);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    for phase in ["implementation_active", "checkpointed", "finish_ready"] {
        baseline_protocol.restore(&protocol_root);
        let mut summary: Value =
            serde_json::from_slice(&fs::read(&path).expect("Summary")).expect("Summary JSON");
        summary["state"] = json!(phase);
        fs::write(&path, serde_json::to_vec(&summary).expect("Summary JSON"))
            .expect("isolated lifecycle projection");
        assert_success(
            &run_cli(&["preflight", "--repo", repo, "--contract", &relative]),
            phase,
        );
    }
}

fn assert_checkpoint_provenance_rejected(case: &str, use_mcp: bool) {
    let root = amended_request_fixture(true);
    let repo = root.path().to_str().expect("repository path");
    let relative = format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json");
    let summary_path = root
        .path()
        .join(format!(".ai/work-items/active/{WORK_ITEM_ID}.summary.json"));
    let contract_path = root.path().join(&relative);
    let decision_path = root.path().join(format!(
        ".ai/decisions/{WORK_ITEM_ID}.preflight-review.json"
    ));
    let material_fallback = case.starts_with("count_");
    if material_fallback {
        // A real non-ancestor HEAD makes committed-material projection
        // unavailable. Keep the declared base and the native journal intact.
        git(
            root.path(),
            &["checkout", "--orphan", "material-unavailable"],
        );
        git(root.path(), &["add", "README.md"]);
        git(
            root.path(),
            &["commit", "-qm", "isolated non-ancestor material HEAD"],
        );
    }
    let read_status = || {
        let result = run_cli(&[
            "work-item",
            "status",
            "--repo",
            repo,
            "--id",
            WORK_ITEM_ID,
            "--json",
        ]);
        assert_success(&result, "checkpoint provenance status");
        serde_json::from_slice::<Value>(&result.stdout).expect("status JSON")
    };
    let control = read_status();
    assert!(
        control["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "untampered native checkpoint must admit review-request generation: {control}"
    );
    if material_fallback {
        assert!(
            control["blockers"]
                .as_array()
                .expect("blockers")
                .contains(&json!("material_review_projection_unavailable")),
            "the real material fallback must be active: {control}"
        );
    }
    let mut summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    match case {
        "missing" | "forged_digest" | "malformed" => {
            let value = match case {
                "forged_digest" => Some(json!(Digest::sha256_bytes(
                    b"same forged checkpoint source"
                ))),
                "malformed" => Some(json!("not-a-contract-digest")),
                _ => None,
            };
            for (stage, field) in [
                ("before_edit", "contractHash"),
                (
                    "contract_amendment_revalidation",
                    "originalBeforeEditContractHash",
                ),
            ] {
                let record = summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoints")
                    .iter_mut()
                    .rfind(|r| r["stage"] == stage)
                    .expect("source checkpoint");
                if let Some(value) = &value {
                    assert_ne!(
                        &record[field], value,
                        "the forged source must differ from native history"
                    );
                    record[field] = value.clone();
                } else {
                    record
                        .as_object_mut()
                        .expect("checkpoint object")
                        .remove(field);
                }
            }
        }
        "count_zero" => summary["checkpointCount"] = json!(0),
        "count_two" => summary["checkpointCount"] = json!(2),
        _ => panic!("unknown checkpoint provenance case"),
    }
    fs::write(
        &summary_path,
        serde_json::to_vec(&summary).expect("Summary JSON"),
    )
    .expect("isolated Summary tamper");
    let before_summary = fs::read(&summary_path).expect("before Summary");
    let before_contract = fs::read(&contract_path).expect("before Contract");
    let before_history = amendment_history_bytes(root.path());
    let before_decision = fs::read(&decision_path).ok();
    let status = read_status();
    let (rejected, response) = if use_mcp {
        let response = mcp_call(
            root.path(),
            &candidate_runtime(),
            "preflight",
            json!({"contract":relative}),
        );
        (
            response.pointer("/result/isError") == Some(&json!(true)),
            response.to_string(),
        )
    } else {
        let response = run_cli(&["preflight", "--repo", repo, "--contract", &relative]);
        (
            !response.status.success(),
            String::from_utf8_lossy(&response.stdout).into_owned(),
        )
    };
    assert!(
        rejected,
        "{case} transport={} must reject request generation; status={status}; response={response}",
        if use_mcp { "MCP" } else { "CLI" }
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "{case} must not regain preflight through material fallback: {status}"
    );
    assert_eq!(
        fs::read(&summary_path).expect("after Summary"),
        before_summary,
        "{case}"
    );
    assert_eq!(
        fs::read(&contract_path).expect("after Contract"),
        before_contract,
        "{case}"
    );
    assert_eq!(
        amendment_history_bytes(root.path()),
        before_history,
        "{case}"
    );
    assert_eq!(fs::read(&decision_path).ok(), before_decision, "{case}");
}

#[test]
fn checkpoint_source_pair_missing_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("missing", false);
}
#[test]
fn checkpoint_source_pair_missing_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("missing", true);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("forged_digest", false);
}
#[test]
fn checkpoint_source_pair_forged_digest_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("forged_digest", true);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("malformed", false);
}
#[test]
fn checkpoint_source_pair_malformed_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("malformed", true);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_zero", false);
}
#[test]
fn checkpoint_count_zero_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_zero", true);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_cli() {
    assert_checkpoint_provenance_rejected("count_two", false);
}
#[test]
fn checkpoint_count_two_material_fallback_is_rejected_by_mcp() {
    assert_checkpoint_provenance_rejected("count_two", true);
}

fn activation_checkpoint_fixture() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("activation fixture directory");
    let root = directory.path().join("repo");
    fs::create_dir(&root).expect("fixture repository");
    let origin = directory.path().join("origin.git");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    git(&root, &["init", "-q", "--initial-branch=main"]);
    git(&root, &["config", "user.name", "Activation fixture"]);
    git(
        &root,
        &["config", "user.email", "activation@example.invalid"],
    );
    fs::write(root.join("README.md"), "isolated activation fixture\n").expect("README");
    git(&root, &["add", "README.md"]);
    git(&root, &["commit", "-qm", "fixture baseline"]);
    git(
        &root,
        &[
            "init",
            "-q",
            "--bare",
            "--initial-branch=main",
            origin.to_str().expect("origin path"),
        ],
    );
    git(
        &origin,
        &["config", "--local", "maintenance.autoDetach", "false"],
    );
    git(&origin, &["config", "--local", "gc.autoDetach", "false"]);
    git(
        &root,
        &[
            "remote",
            "add",
            "origin",
            origin.to_str().expect("origin path"),
        ],
    );
    git(&root, &["push", "-q", "-u", "origin", "main"]);
    git(&root, &["remote", "set-head", "origin", "-a"]);
    git(&root, &["checkout", "-q", "-b", "fixture/activation"]);
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(&cli(&["attach"]), "attach activation fixture");
    assert_success(
        &cli(&["work-item", "new", "--id", id, "--mode", "implementation"]),
        "create real not_ready scaffold",
    );
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("scaffold"))
        .expect("scaffold JSON");
    assert_eq!(contract["state"], "not_ready");
    let intent = "Exercise a real native activation boundary";
    let goal = "Keep post-checkpoint amendments reviewable";
    let acceptance = "The activation boundary preserves strict native amendment admission";
    let request_path = directory.path().join("before-activation.json");
    fs::write(
        &request_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "before-activation",
            "expectedContractDigest": cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason": "Isolated test plan; no real Work Item or human authority.",
            "changes": [
                {"path":"/intent","operation":"set","value":intent},
                {"path":"/goal","operation":"set","value":goal},
                {"path":"/scope","operation":"replace","value":["README.md"]},
                {"path":"/outOfScope","operation":"replace","value":[]},
                {"path":"/risk","operation":"set","value":"normal"},
                {"path":"/authority","operation":"set","value":"authorized"},
                {"path":"/acceptanceCriteria","operation":"replace","value":[acceptance]},
                {"path":"/verification","operation":"replace","value":["git diff --check"]}
            ]
        }))
        .expect("request JSON"),
    )
    .expect("pre-activation request");
    assert_success(
        &cli(&[
            "work-item",
            "amend",
            "--id",
            id,
            "--request",
            request_path.to_str().expect("request path"),
        ]),
        "record pre-activation amendment",
    );
    assert_success(
        &cli(&[
            "start",
            "--id",
            id,
            "--intent",
            intent,
            "--goal",
            goal,
            "--scope",
            "README.md",
            "--risk",
            "normal",
            "--authority",
            "authorized",
            "--acceptance",
            acceptance,
            "--verification",
            "git diff --check",
            "--prepare",
        ]),
        "activate real scaffold and prepare review",
    );
    let active: Value = serde_json::from_slice(&fs::read(&contract_path).expect("active Contract"))
        .expect("active JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("fixture-review.json");
    fs::write(
        &review_path,
        serde_json::to_vec_pretty(&json!({"decisionEvidence": {
            "schemaVersion": 1,
            "decisionId": "contract-preflight-review",
            "decision": "confirm_review",
            "workItemId": id,
            "repositoryId": active["repositoryId"],
            "contractDigest": cockpit_protocol::digest_json(&active).expect("active digest"),
            "preflightDecisionDigest": summary["preflightDecisionDigest"],
            "repositorySnapshotDigest": summary["preflightRepositorySnapshotDigest"],
            "recordedAt": "2026-10-17T00:00:00Z",
            "recordedBy": "fixture:activation-test-only",
            "reason": "Synthetic isolated fixture review; not a real human response or history proof."
        }}))
        .expect("fixture review JSON"),
    )
    .expect("fixture review");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            review_path.to_str().expect("review path"),
        ]),
        "record fixture-only review",
    );
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("Contract path"),
        ]),
        "preflight reviewed fixture",
    );
    assert_success(&cli(&["checkpoint", "--id", id]), "fixture checkpoint");
    let post_path = directory.path().join("after-checkpoint.json");
    fs::write(
        &post_path,
        serde_json::to_vec_pretty(&json!({
            "schemaVersion": 1,
            "changeId": "after-checkpoint",
            "expectedContractDigest": cockpit_protocol::digest_json(&active).expect("digest"),
            "reason": "Keep the amended fixture plan reviewable without granting its review.",
            "changes": [{"path":"/goal","operation":"set","value":"Review the current amended fixture plan"}]
        }))
        .expect("post-checkpoint JSON"),
    )
    .expect("post-checkpoint request");
    let amended = cli(&[
        "work-item",
        "amend",
        "--id",
        id,
        "--request",
        post_path.to_str().expect("post request path"),
    ]);
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    let history = cli(&["work-item", "amendments", "--id", id]);
    assert_success(&history, "read intact amendment journal");
    let history: Vec<Value> = serde_json::from_slice(&history.stdout).expect("history JSON");
    assert_eq!(history.len(), 2);
    assert_eq!(
        history[1]["previousJournalDigest"],
        history[0]["journalDigest"]
    );
    if !amended.status.success() || !status.status.success() {
        let preserved = directory.keep();
        panic!(
            "real activation boundary must keep amendment admission reviewable; fixture preserved at {}; amendment stderr={}; status stderr={}",
            preserved.display(),
            String::from_utf8_lossy(&amended.stderr),
            String::from_utf8_lossy(&status.stderr)
        );
    }
    let status: Value = serde_json::from_slice(&status.stdout).expect("current status JSON");
    assert_eq!(status["humanDecisionRequired"], true, "{status}");
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("safe actions")
            .iter()
            .any(|action| action == "run_verification"),
        "the new Contract still needs its own human review: {status}"
    );
    directory
}

#[test]
fn native_activation_keeps_post_checkpoint_amendment_reviewable() {
    activation_checkpoint_fixture();
}

#[test]
fn current_contract_review_cannot_hide_a_missing_activation_commit() {
    let directory = activation_checkpoint_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("native fixture command")
    };
    assert_success(
        &cli(&[
            "preflight",
            "--contract",
            contract_path.to_str().expect("path"),
        ]),
        "prepare current Contract review",
    );
    let current: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    let review_path = directory.path().join("current-fixture-review.json");
    fs::write(&review_path, serde_json::to_vec(&json!({"decisionEvidence": {
        "schemaVersion":1, "decisionId":"contract-preflight-review", "decision":"confirm_review",
        "workItemId":id, "repositoryId":current["repositoryId"],
        "contractDigest":cockpit_protocol::digest_json(&current).expect("digest"),
        "preflightDecisionDigest":summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
        "recordedAt":"2026-10-17T00:00:00Z", "recordedBy":"fixture:current-review-only",
        "reason":"Synthetic current Contract review never authenticates a missing activation phase."
    }})).expect("review JSON")).expect("review source");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            review_path.to_str().expect("path"),
        ]),
        "record current fixture review",
    );
    let before_contract = fs::read(&contract_path).expect("Contract bytes");
    let before_summary = fs::read(&summary_path).expect("Summary bytes");
    fs::remove_file(root.join(format!(
        ".ai/evidence/{id}.activation-transition/committed.json"
    )))
    .expect("simulate isolated partial publication");
    let result = cli(&["work-item", "status", "--id", id, "--json"]);
    assert!(
        !result.status.success(),
        "current review must not hide missing activation commit: {}",
        String::from_utf8_lossy(&result.stdout)
    );
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let preflight = cli(&[
        "preflight",
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    assert!(
        !preflight.status.success(),
        "current preflight must reject the same missing activation commit"
    );
    assert!(
        String::from_utf8_lossy(&preflight.stderr).contains("unresolved activation transition"),
        "{}",
        String::from_utf8_lossy(&preflight.stderr)
    );
    assert_eq!(
        fs::read(contract_path).expect("Contract unchanged"),
        before_contract
    );
    assert_eq!(
        fs::read(summary_path).expect("Summary unchanged"),
        before_summary
    );
}

fn legacy_activation_fixture() -> tempfile::TempDir {
    let directory = activation_checkpoint_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    // This isolated fixture models the old on-disk format. Removing its new
    // proof does not authenticate the historical transition or any real WI.
    fs::remove_dir_all(root.join(format!(".ai/evidence/{id}.activation-transition")))
        .expect("model a legacy fixture without a forward receipt");
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let mut legacy_summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("Summary")).expect("Summary JSON");
    legacy_summary
        .as_object_mut()
        .expect("Summary object")
        .remove("activationContinuity");
    fs::write(
        &summary_path,
        serde_json::to_vec(&legacy_summary).expect("legacy Summary JSON"),
    )
    .expect("model the legacy Summary format without a capability marker");
    let contract: Value = serde_json::from_slice(&fs::read(&contract_path).expect("Contract"))
        .expect("Contract JSON");
    let runtime = candidate_runtime();
    let head = git_output(&root, &["rev-parse", "HEAD"]);
    assert!(
        head.status.success(),
        "fixture HEAD: {}",
        String::from_utf8_lossy(&head.stderr)
    );
    let branch = git_output(&root, &["branch", "--show-current"]);
    assert!(
        branch.status.success(),
        "fixture branch: {}",
        String::from_utf8_lossy(&branch.stderr)
    );
    let input = directory.path().join("registration.json");
    fs::write(&input, serde_json::to_vec(&json!({
        "schemaVersion":1, "repositoryId":contract["repositoryId"], "workItemId":id,
        "contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "worktreePath":root.canonicalize().expect("root").to_string_lossy(),
        "branch":String::from_utf8(branch.stdout).expect("branch UTF8").trim(),
        "head":String::from_utf8(head.stdout).expect("head UTF8").trim(), "generation":1,
        "runtime":{"schemaVersion":1,"runtimeVersion":runtime.runtime_version,
            "runtimeDigest":runtime.runtime_digest,"capability":"observed_environment_drift_v1"},
        "declaration":{"providedOutcomes":[],"consumedOutcomes":[],"resourceClaims":[],
            "integrationResponsibility":{"responsibleWorkItemId":id,"targetBranch":"main",
                "compositionOrder":[],"rationale":"Isolated recovery fixture; no real authority."},
            "compositionVerification":{"compatibilityConstraints":[],"requiredScenarios":[],"reusableNodes":[]}}
    })).expect("registration JSON")).expect("registration input");
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("fixture CLI")
    };
    assert_success(
        &cli(&[
            "work-item",
            "coordination",
            "register",
            "--input",
            input.to_str().expect("path"),
        ]),
        "register exact legacy fixture owner",
    );
    directory
}

fn assert_legacy_activation_recovery_flow(record_via_mcp: bool, existing_c_review: bool) {
    let directory = legacy_activation_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
    let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
    let runtime = candidate_runtime();
    let cli = |args: &[&str]| {
        Command::new(candidate_binary())
            .args(args)
            .arg("--repo")
            .arg(&root)
            .output()
            .expect("fixture CLI")
    };
    if existing_c_review {
        // Model a previously stored self-declared C receipt in this isolated
        // legacy-format fixture. It is not a real human or historical proof.
        let mut review: Value = serde_json::from_slice(
            &fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json"))).unwrap(),
        )
        .unwrap();
        let contract: Value = serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
        review["contractDigest"] = json!(cockpit_protocol::digest_json(&contract).unwrap());
        review["preflightDecisionDigest"] =
            json!(Digest::sha256_bytes(b"synthetic old C preflight"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
        summary["decisionEvidence"] = review.clone();
        summary["preflightDecisionDigest"] = review["preflightDecisionDigest"].clone();
        summary["preflightContractDigest"] = review["contractDigest"].clone();
        summary["preflightRepositorySnapshotDigest"] = review["repositorySnapshotDigest"].clone();
        summary["preflightState"] = json!("green");
        fs::write(&summary_path, serde_json::to_vec(&summary).unwrap()).unwrap();
        fs::write(
            root.join(format!(".ai/decisions/{id}.preflight-review.json")),
            serde_json::to_vec(&review).unwrap(),
        )
        .unwrap();
    }
    let before_contract = fs::read(&contract_path).expect("Contract bytes");
    let before_summary = fs::read(&summary_path).expect("Summary bytes");
    let planned = cli(&["work-item", "activation-recovery", "plan", "--id", id]);
    assert_success(&planned, "plan explicit present-baseline recovery");
    let request: Value = serde_json::from_slice(&planned.stdout).expect("recovery request");
    assert_eq!(request["historicalTransitionAuthentication"], "unknown");
    assert_eq!(request["businessAuthorityGranted"], false);
    assert_eq!(request["freshContractReviewRequired"], true);
    assert_eq!(request["requiredResponse"], "accept_present_baseline");
    assert_eq!(request["registrationGeneration"], 1);
    let mcp = mcp_call(
        &root,
        &runtime,
        "work_item_activation_recovery_plan",
        json!({"workItemId":id}),
    );
    assert_ne!(mcp.pointer("/result/isError"), Some(&json!(true)), "{mcp}");
    let mcp_request: Value = serde_json::from_str(
        mcp["result"]["content"][0]["text"]
            .as_str()
            .expect("MCP text"),
    )
    .expect("MCP request");
    assert_eq!(request, mcp_request);
    assert_eq!(
        fs::read(&contract_path).expect("read Contract"),
        before_contract
    );
    assert_eq!(
        fs::read(&summary_path).expect("read Summary"),
        before_summary
    );

    let relative_source = format!(".ai/evidence/{id}/present-response.json");
    let source_path = root.join(&relative_source);
    fs::create_dir_all(source_path.parent().expect("parent")).expect("evidence directory");
    let source = json!({"schemaVersion":1,"kind":"human_activation_recovery_response_source",
        "requestDigest":request["requestDigest"],"response":"accept_present_baseline",
        "humanActor":"human:isolated-test-fixture","respondedAt":"2026-10-17T00:00:00Z",
        "provenance":"Synthetic fixture response only; no real human or historical authentication."});
    let source_bytes = serde_json::to_vec(&source).expect("source JSON");
    fs::write(&source_path, &source_bytes).expect("fixture-only response source");
    let decision = json!({"schemaVersion":1,"decisionId":"activation-present-baseline-review",
        "response":"accept_present_baseline","requestDigest":request["requestDigest"],
        "humanActor":"human:isolated-test-fixture","recorderActor":"agent:test-fixture",
        "assurance":"self_declared","authoritySourcePath":relative_source,
        "authoritySourceDigest":Digest::sha256_bytes(&source_bytes),
        "respondedAt":"2026-10-17T00:00:00Z","reason":"Review only the exact present fixture baseline and retained unknowns."});
    let decision_path = directory.path().join("decision.json");
    let mut old_review = decision.clone();
    old_review["response"] = json!("confirm_review");
    fs::write(
        &decision_path,
        serde_json::to_vec(&old_review).expect("old review JSON"),
    )
    .expect("invalid decision input");
    let rejected = cli(&[
        "work-item",
        "activation-recovery",
        "record",
        "--id",
        id,
        "--input",
        decision_path.to_str().expect("path"),
    ]);
    assert!(
        !rejected.status.success(),
        "old B-style review cannot recover the gap"
    );
    assert_eq!(
        fs::read(&summary_path).expect("Summary unchanged"),
        before_summary
    );
    assert!(
        !root
            .join(format!(".ai/evidence/{id}.activation-recovery"))
            .exists()
    );
    fs::write(
        &decision_path,
        serde_json::to_vec(&decision).expect("decision JSON"),
    )
    .expect("decision input");
    let record: Value = if record_via_mcp {
        let recorded = mcp_call(
            &root,
            &runtime,
            "work_item_activation_recovery_record",
            json!({"workItemId":id,"decision":decision}),
        );
        assert_eq!(
            recorded.pointer("/result/isError"),
            Some(&json!(false)),
            "{recorded}"
        );
        serde_json::from_str(
            recorded["result"]["content"][0]["text"]
                .as_str()
                .expect("record text"),
        )
        .expect("MCP record JSON")
    } else {
        let recorded = cli(&[
            "work-item",
            "activation-recovery",
            "record",
            "--id",
            id,
            "--input",
            decision_path.to_str().expect("path"),
        ]);
        assert_success(&recorded, "record exact present fixture baseline");
        serde_json::from_slice(&recorded.stdout).expect("CLI record JSON")
    };
    assert_eq!(record["request"], request);
    assert_eq!(record["historicalTransitionAuthentication"], "unknown");
    assert_eq!(record["businessAuthorityGranted"], false);
    assert_eq!(record["freshContractReviewRequired"], true);
    let before_duplicate = recovery_metadata_bytes(&root);
    let mut duplicate = decision.clone();
    duplicate["reason"] = json!("A different decision cannot overwrite an accepted baseline");
    let duplicate = mcp_call(
        &root,
        &runtime,
        "work_item_activation_recovery_record",
        json!({"workItemId":id,"decision":duplicate}),
    );
    assert_eq!(
        duplicate.pointer("/result/isError"),
        Some(&json!(true)),
        "{duplicate}"
    );
    assert_eq!(recovery_metadata_bytes(&root), before_duplicate);
    assert_eq!(
        fs::read(&contract_path).expect("Contract remains unchanged"),
        before_contract
    );
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    assert_success(&status, "read recovered metadata status");
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert!(
        status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_preflight")),
        "{status}"
    );
    assert!(
        !status["safeActions"]
            .as_array()
            .expect("actions")
            .contains(&json!("run_verification")),
        "{status}"
    );
    let preflight = cli(&[
        "preflight",
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    assert_success(&preflight, "generate fresh current Contract review request");
    let preflight: Value = serde_json::from_slice(&preflight.stdout).expect("preflight JSON");
    assert_eq!(
        preflight["reviewState"], "needs_human_confirmation",
        "{preflight}"
    );
    let fresh_summary: Value =
        serde_json::from_slice(&fs::read(&summary_path).expect("fresh Summary"))
            .expect("fresh Summary JSON");
    let contract: Value = serde_json::from_slice(&before_contract).expect("current Contract");
    let fresh_review = directory.path().join("post-recovery-fixture-review.json");
    fs::write(&fresh_review, serde_json::to_vec(&json!({"decisionEvidence": {
        "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
        "workItemId":id,"repositoryId":contract["repositoryId"],
        "contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "preflightDecisionDigest":fresh_summary["preflightDecisionDigest"],
        "repositorySnapshotDigest":fresh_summary["preflightRepositorySnapshotDigest"],
        "recordedAt":record["recordedAt"],"recordedBy":"fixture:separate-post-recovery-review",
        "reason":"Separate synthetic current-Contract review after the recovery request; no real human approval."
    }})).expect("fresh review JSON")).expect("fresh review input");
    assert_success(
        &cli(&[
            "work-item",
            "controls",
            "--id",
            id,
            "--input",
            fresh_review.to_str().expect("path"),
        ]),
        "record separate current fixture review",
    );
    assert_success(
        &cli(&["verify", "--work-item", id, "--workers", "1"]),
        "only the separately reviewed fixture may verify",
    );
    let status = cli(&["work-item", "status", "--id", id, "--json"]);
    assert_success(
        &status,
        "read status after native verification projection refresh",
    );
    let status: Value = serde_json::from_slice(&status.stdout).expect("status JSON");
    assert_eq!(
        status["humanDecisionRequired"], false,
        "a native verification refresh must not create a review loop: {status}"
    );
    assert!(
        !status["blockers"]
            .as_array()
            .expect("blockers")
            .contains(&json!("activation_recovery_fresh_contract_review_required")),
        "{status}"
    );
    assert!(
        status["unknowns"]
            .as_array()
            .expect("unknowns")
            .contains(&json!("historical_transition_authentication_unknown")),
        "separate C review must not erase the missing historical authentication: {status}"
    );
    let outcome = cli(&["work-item", "outcome", "--id", id, "--json"]);
    assert_success(&outcome, "read honest post-recovery Outcome");
    let outcome: Value = serde_json::from_slice(&outcome.stdout).expect("Outcome JSON");
    assert!(
        outcome["unknowns"]
            .as_array()
            .expect("Outcome unknowns")
            .contains(&json!("historical_transition_authentication_unknown")),
        "{outcome}"
    );
    fs::remove_file(root.join(format!(
        ".ai/evidence/{id}.activation-recovery/committed.json"
    )))
    .expect("model a valid typed prepared-only recovery phase");
    let before_partial = recovery_metadata_bytes(&root);
    let partial = cli(&["work-item", "status", "--id", id, "--json"]);
    assert!(
        !partial.status.success(),
        "reviewed and verified C cannot hide a partial recovery phase"
    );
    assert!(
        String::from_utf8_lossy(&partial.stderr)
            .contains("unresolved activation recovery transaction"),
        "{}",
        String::from_utf8_lossy(&partial.stderr)
    );
    assert_eq!(recovery_metadata_bytes(&root), before_partial);
    let outcome = cli(&["work-item", "outcome", "--id", id, "--json"]);
    assert!(
        !outcome.status.success(),
        "a partial recovery cannot render an apparently complete Outcome"
    );
    assert_eq!(recovery_metadata_bytes(&root), before_partial);
}

#[test]
fn legacy_activation_requires_an_exact_present_baseline_decision_and_fresh_review() {
    assert_legacy_activation_recovery_flow(false, false);
}

#[test]
fn legacy_activation_mcp_record_requires_a_separate_current_contract_review() {
    assert_legacy_activation_recovery_flow(true, false);
}

#[test]
fn a_preexisting_c_receipt_cannot_preapprove_present_baseline_recovery() {
    assert_legacy_activation_recovery_flow(false, true);
}

fn recovery_metadata_bytes(root: &Path) -> Vec<(PathBuf, Vec<u8>)> {
    fn walk(path: &Path, values: &mut Vec<(PathBuf, Vec<u8>)>) {
        if !path.exists() {
            return;
        }
        for entry in fs::read_dir(path).expect("metadata directory") {
            let entry = entry.expect("metadata entry");
            if entry.file_type().expect("entry type").is_dir() {
                walk(&entry.path(), values);
            } else {
                values.push((
                    entry.path(),
                    fs::read(entry.path()).expect("metadata bytes"),
                ));
            }
        }
    }
    let mut values = Vec::new();
    walk(&root.join(".ai"), &mut values);
    walk(&root.join(".git/.ai-cockpit"), &mut values);
    values.sort_by(|left, right| left.0.cmp(&right.0));
    values
}

#[test]
fn invalid_legacy_recovery_plans_preserve_all_metadata_bytes() {
    let directory = legacy_activation_fixture();
    let baseline = FixtureTreeSnapshot::capture(directory.path());
    for case in [
        "missing_start",
        "missing_checkpoint",
        "forged_checkpoint",
        "pending_amendment",
        "partial_recovery",
        "foreign_owner",
        "foreign_runtime",
        "unsupported_runtime",
        "new_forward_marker",
        "orphan_recovery_marker",
    ] {
        baseline.restore(directory.path());
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let mut summary: Value = serde_json::from_slice(&fs::read(&summary_path).expect("Summary"))
            .expect("Summary JSON");
        match case {
            "missing_start" => {
                summary["lifecycleFacts"]
                    .as_object_mut()
                    .expect("facts")
                    .remove("start");
            }
            "missing_checkpoint" => {
                summary["checkpointEvidence"] = json!([]);
            }
            "forged_checkpoint" => {
                for record in summary["checkpointEvidence"]
                    .as_array_mut()
                    .expect("checkpoint records")
                {
                    if record["stage"] == "before_edit" {
                        record["contractHash"] = json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                    if record["stage"] == "contract_amendment_revalidation" {
                        record["originalBeforeEditContractHash"] =
                            json!(Digest::sha256_bytes(b"forged checkpoint"));
                    }
                }
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .expect("model pending fixture");
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).expect("partial directory");
                fs::write(partial.join("prepared.json"), b"{}").expect("partial fixture phase");
            }
            "foreign_owner" => {
                git(&root, &["checkout", "-qb", "foreign-fixture-owner"]);
            }
            "new_forward_marker" => {
                summary["activationContinuity"] = json!({"capability":"activation_continuity_recovery_v1","journalRelevant":true});
            }
            "orphan_recovery_marker" => {
                summary["activationRecovery"] = json!({"schemaVersion":1,
                    "requestDigest":Digest::sha256_bytes(b"missing recovery record"),
                    "historicalTransitionAuthentication":"unknown","businessAuthorityGranted":false,
                    "freshContractReviewRequired":true});
            }
            _ => {}
        }
        fs::write(
            &summary_path,
            serde_json::to_vec(&summary).expect("fixture Summary JSON"),
        )
        .expect("fixture mutation");
        let before = recovery_metadata_bytes(&root);
        let mut runtime = candidate_runtime();
        if case == "foreign_runtime" {
            runtime.runtime_digest = Digest::sha256_bytes(b"foreign fixture Runtime");
        }
        if case == "unsupported_runtime" {
            runtime.runtime_version = "0.2.105".into();
        }
        let response = mcp_call(
            &root,
            &runtime,
            "work_item_activation_recovery_plan",
            json!({"workItemId":id}),
        );
        if response.pointer("/result/isError") != Some(&json!(true)) {
            let preserved = directory.keep();
            panic!(
                "{case}: unexpected request; fixture preserved at {}; {response}",
                preserved.display()
            );
        }
        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case} must not repair or publish metadata implicitly"
        );
    }
}

#[test]
fn recovery_request_binds_native_environment_and_raw_review_bytes() {
    let directory = legacy_activation_fixture();
    let root = directory.path().join("repo");
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let planned = run_cli(&[
        "work-item",
        "activation-recovery",
        "plan",
        "--id",
        id,
        "--repo",
        root.to_str().expect("root"),
    ]);
    assert_success(&planned, "plan bound metadata request");
    let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
    let observed = cockpit_repository::RepositoryExecutionContext::capture(&root)
        .expect("context")
        .observe_environment_identity()
        .expect("actual environment");
    assert_eq!(
        request["environmentDigest"],
        json!(observed.environment_digest)
    );
    let review = fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
        .expect("historical review bytes");
    assert_eq!(
        request["rawEvidenceDigests"]["preflight-review"],
        json!(Digest::sha256_bytes(&review))
    );
}

fn fixture_present_recovery_decision(root: &Path, request: &Value) -> Value {
    let id = "WI-CLI-ACTIVATION-CONTINUITY";
    let relative = format!(".ai/evidence/{id}/actual-fixture-response.json");
    let path = root.join(&relative);
    fs::create_dir_all(path.parent().expect("response parent")).expect("response directory");
    let source = json!({"schemaVersion":1,"kind":"human_activation_recovery_response_source",
        "requestDigest":request["requestDigest"],"response":"accept_present_baseline",
        "humanActor":"human:isolated-test-fixture","respondedAt":"2026-10-17T00:00:00Z",
        "provenance":"Synthetic response in an isolated test only; no real human authentication."});
    let bytes = serde_json::to_vec(&source).expect("response JSON");
    fs::write(path, &bytes).expect("fixture response source");
    json!({"schemaVersion":1,"decisionId":"activation-present-baseline-review",
        "response":"accept_present_baseline","requestDigest":request["requestDigest"],
        "humanActor":"human:isolated-test-fixture","recorderActor":"agent:test-fixture",
        "assurance":"self_declared","authoritySourcePath":relative,
        "authoritySourceDigest":Digest::sha256_bytes(&bytes),"respondedAt":"2026-10-17T00:00:00Z",
        "reason":"Accept only the exact isolated present baseline, retaining all historical unknowns."})
}

#[test]
fn stale_or_substituted_recovery_decisions_never_publish_or_finish_metadata() {
    let directory = legacy_activation_fixture();
    let baseline = FixtureTreeSnapshot::capture(directory.path());
    for case in [
        "request",
        "summary",
        "contract",
        "history",
        "head",
        "registration",
        "environment",
        "source",
        "agent_as_human",
        "blank_human",
        "old_review_source",
        "partial_recovery",
        "pending_amendment",
        "unknown_field",
    ] {
        baseline.restore(directory.path());
        let root = directory.path().join("repo");
        let id = "WI-CLI-ACTIVATION-CONTINUITY";
        let planned = run_cli(&[
            "work-item",
            "activation-recovery",
            "plan",
            "--id",
            id,
            "--repo",
            root.to_str().expect("root"),
        ]);
        assert_success(&planned, "plan current recovery fixture");
        let request: Value = serde_json::from_slice(&planned.stdout).expect("request");
        let mut decision = fixture_present_recovery_decision(&root, &request);
        let summary_path = root.join(format!(".ai/work-items/active/{id}.summary.json"));
        let contract_path = root.join(format!(".ai/work-items/active/{id}.contract.json"));
        let source_path = root.join(
            decision["authoritySourcePath"]
                .as_str()
                .expect("source path"),
        );
        match case {
            "request" => {
                decision["requestDigest"] = json!(Digest::sha256_bytes(b"another request"));
            }
            "summary" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&summary_path).unwrap()).unwrap();
                value["fixtureNote"] = json!("changed after request");
                fs::write(&summary_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "contract" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&contract_path).unwrap()).unwrap();
                value["goal"] = json!("a different current Contract");
                fs::write(&contract_path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "history" => {
                let path = root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000001.prepared.json"
                ));
                let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
                value["prepared"]["request"]["reason"] = json!("changed raw history after request");
                fs::write(path, serde_json::to_vec(&value).unwrap()).unwrap();
            }
            "head" => {
                git(
                    &root,
                    &[
                        "commit",
                        "--allow-empty",
                        "-qm",
                        "advance isolated fixture HEAD",
                    ],
                );
            }
            "registration" => {
                let input = directory.path().join("registration.json");
                let mut value: Value = serde_json::from_slice(&fs::read(&input).unwrap()).unwrap();
                value["generation"] = json!(2);
                fs::write(&input, serde_json::to_vec(&value).unwrap()).unwrap();
                let registered = run_cli(&[
                    "work-item",
                    "coordination",
                    "register",
                    "--repo",
                    root.to_str().unwrap(),
                    "--input",
                    input.to_str().unwrap(),
                ]);
                assert_success(&registered, "advance actual native fixture registration");
            }
            "source" => {
                fs::write(&source_path, b"{}").unwrap();
            }
            "agent_as_human" | "blank_human" => {
                let mut value: Value =
                    serde_json::from_slice(&fs::read(&source_path).unwrap()).unwrap();
                value["humanActor"] = if case == "blank_human" {
                    json!("human:   ")
                } else {
                    json!("agent:cannot-supply-human-decision")
                };
                let bytes = serde_json::to_vec(&value).unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["humanActor"] = value["humanActor"].clone();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "old_review_source" => {
                let bytes =
                    fs::read(root.join(format!(".ai/decisions/{id}.preflight-review.json")))
                        .unwrap();
                fs::write(&source_path, &bytes).unwrap();
                decision["authoritySourceDigest"] = json!(Digest::sha256_bytes(&bytes));
            }
            "partial_recovery" => {
                let partial = root.join(format!(".ai/evidence/{id}.activation-recovery"));
                fs::create_dir(&partial).unwrap();
                fs::write(partial.join("prepared.json"), b"{}").unwrap();
            }
            "pending_amendment" => {
                fs::remove_file(root.join(format!(
                    ".ai/evidence/{id}.contract-amendments/00000002.committed.json"
                )))
                .unwrap();
            }
            "unknown_field" => {
                decision["unsupported"] = json!(true);
            }
            _ => {}
        }
        let input = directory.path().join("recovery-decision.json");
        fs::write(&input, serde_json::to_vec(&decision).unwrap()).unwrap();
        let before = recovery_metadata_bytes(&root);
        if case == "unknown_field" {
            let response = mcp_call(
                &root,
                &candidate_runtime(),
                "work_item_activation_recovery_record",
                json!({"workItemId":id,"decision":decision}),
            );
            assert_eq!(
                response.pointer("/result/isError"),
                Some(&json!(true)),
                "{case}: {response}"
            );
        } else {
            let mut command = Command::new(candidate_binary());
            command.args([
                "work-item",
                "activation-recovery",
                "record",
                "--repo",
                root.to_str().unwrap(),
                "--id",
                id,
                "--input",
                input.to_str().unwrap(),
            ]);
            if case == "environment" {
                let original = std::env::var_os("CARGO_BUILD_JOBS");
                let changed = if original.as_deref() == Some(std::ffi::OsStr::new("2")) {
                    "1"
                } else {
                    "2"
                };
                assert_ne!(original.as_deref(), Some(std::ffi::OsStr::new(changed)));
                command.env("CARGO_BUILD_JOBS", changed);
            } // Metadata-only rejection: no build is launched.
            let output = command.output().expect("record fixture decision");
            if output.status.success() {
                let preserved = directory.keep();
                panic!(
                    "{case}: unexpected recovery publication; fixture preserved at {}; response bytes={}, raw digest={}",
                    preserved.display(),
                    output.stdout.len(),
                    Digest::sha256_bytes(&output.stdout)
                );
            }
        }

        assert_eq!(
            recovery_metadata_bytes(&root),
            before,
            "{case}: refusal must preserve every existing phase and byte"
        );
    }
}

fn confirm_delegated_fixture_owner(root: &Path, wi: &str) {
    let active = root.join(".ai/work-items/active");
    let contract: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{wi}.contract.json"))).expect("Contract"),
    )
    .expect("JSON");
    let summary: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{wi}.summary.json"))).expect("Summary"),
    )
    .expect("JSON");
    record_work_item_governance_controls_with_runtime(root, wi, &json!({"intentAlignment":{"state":"unknown","evidence":[],"reason":"isolated bounded flow is planned, not yet validated or measured"},"decisionEvidence": {
        "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
        "workItemId":wi,"repositoryId":repository_id(root),"contractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
        "preflightDecisionDigest":summary["preflightDecisionDigest"],"repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
        "recordedAt":"2026-10-23T00:00:00Z","recordedBy":"human:synthetic-isolated-fixture",
        "reason":"synthetic native owner baseline, no real external identity authentication"
    }}), &candidate_runtime()).expect("synthetic owner review");
}

#[test]
fn delegated_cli_mcp_flow_executes_checks_after_one_synthetic_baseline_review() {
    let root = repository();
    let root_path: &::std::path::Path = root.path();
    let wi = "WI-CLI-DELEGATED-FLOW";
    let policy = json!({"schemaVersion":1,"reviewerActor":"agent:fixture","authoritySource":"user:synthetic-flow",
        "allowedChanges":["append_verification"],"preapprovedCommands":["git status --porcelain", "git ls-files --error-unmatch retry-fixture.txt"]});
    let input = tempfile::NamedTempFile::new().expect("policy file");
    fs::write(
        input.path(),
        serde_json::to_vec(&policy).expect("policy JSON"),
    )
    .expect("input");
    let start = run_cli(&[
        "start",
        "--repo",
        root_path.to_str().expect("path"),
        "--id",
        wi,
        "--intent",
        "isolated bounded development",
        "--goal",
        "one baseline review",
        "--scope",
        "**",
        "--authority",
        "authorized",
        "--acceptance",
        "exact checks execute",
        "--verification",
        "git diff --check",
        "--development-review",
        input.path().to_str().expect("input path"),
        "--prepare",
    ]);
    assert_success(&start, "opt-in prepared start");
    let started: Value = serde_json::from_slice(&start.stdout).expect("start JSON");
    assert!(
        started["preflight"]["humanDecisionRequest"].is_object(),
        "{started}"
    );
    confirm_delegated_fixture_owner(root_path, wi);
    let contract_path = root_path.join(format!(".ai/work-items/active/{wi}.contract.json"));
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            root_path.to_str().expect("path"),
            "--contract",
            contract_path.to_str().expect("Contract path"),
        ]),
        "reviewed preflight",
    );
    assert_success(
        &run_cli(&[
            "checkpoint",
            "--repo",
            root_path.to_str().expect("path"),
            "--id",
            wi,
        ]),
        "checkpoint",
    );
    fs::create_dir_all(root_path.join("docs/work-items")).expect("fixture docs");
    fs::create_dir_all(root_path.join("docs/reference")).expect("fixture parity");
    for suffix in ["", ".zh-CN", ".ja"] {
        fs::write(root_path.join(format!("docs/work-items/{wi}{suffix}.md")),format!("---\nworkItemId: {wi}\nlastVerifiedBy: {wi}\nstatus: in_progress\n---\nSynthetic bounded-development fixture; no delivered capability.\n")).expect("required fixture projection");
        fs::write(
            root_path.join(format!("docs/reference/reference-parity{suffix}.md")),
            format!("| {wi} | In progress | synthetic fixture |\n"),
        )
        .expect("required fixture parity");
    }

    git(root_path, &["add", "docs"]);
    git(
        root_path,
        &["commit", "-qm", "in-scope fixture documentation"],
    );
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            root_path.to_str().expect("path"),
            "--contract",
            contract_path.to_str().expect("path"),
        ]),
        "fresh preflight after in-scope documentation work",
    );

    let before: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract")).expect("JSON");
    let owner_path = root_path.join(format!(".ai/decisions/{wi}.preflight-review.json"));
    let owner_bytes = fs::read(&owner_path).expect("owner receipt");
    let amended = mcp_call(
        root_path,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId":wi,"request":{
            "schemaVersion":1,"changeId":"exact-preapproved-command","expectedContractDigest":cockpit_protocol::digest_json(&before).expect("digest"),
            "reason":"add the exact owner-preapproved check","changes":[{"path":"/verification","operation":"add","value":"git status --porcelain"}],
            "developmentReview":{"actor":"agent:fixture","authoritySource":"user:synthetic-flow","assurance":"self_declared","operations":["review_development_amendment"],"policyRefs":[],"evidenceRefs":[]}
        }}),
    );
    assert_eq!(
        amended.pointer("/result/isError"),
        Some(&json!(false)),
        "{amended}"
    );
    let output = run_cli(&[
        "preflight",
        "--repo",
        root_path.to_str().expect("path"),
        "--contract",
        contract_path.to_str().expect("path"),
    ]);
    if !output.status.success() {
        let status = run_cli(&[
            "work-item",
            "status",
            "--repo",
            root_path.to_str().expect("path"),
            "--id",
            wi,
            "--json",
        ]);
        eprintln!(
            "actual failed-flow status: {}",
            String::from_utf8_lossy(&status.stdout)
        );
    }
    assert_success(&output, "technical preflight");
    let report: Value = serde_json::from_slice(&output.stdout).expect("preflight JSON");
    assert!(report["humanDecisionRequest"].is_null(), "{report}");
    let output = run_cli(&[
        "verify",
        "--repo",
        root_path.to_str().expect("path"),
        "--work-item",
        wi,
    ]);
    assert_success(&output, "fresh delegated verify");
    let attempt_dir = root_path.join(".ai/evidence");
    let attempts: Vec<Value> = fs::read_dir(&attempt_dir)
        .expect("evidence")
        .map(|e| e.expect("entry").path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{wi}.verification-attempt.")))
        })
        .map(|p| serde_json::from_slice(&fs::read(p).expect("attempt")).expect("attempt JSON"))
        .collect();
    assert_eq!(attempts.len(), 1);
    assert_eq!(attempts[0]["state"], "execution_completed");
    assert_eq!(
        attempts[0]["commands"]
            .as_array()
            .expect("actual commands")
            .len(),
        2
    );
    assert_eq!(attempts[0]["receipt"]["nodesExecuted"], 2);
    assert_eq!(attempts[0]["receipt"]["nodesReused"], 0);
    assert_eq!(attempts[0]["receipt"]["processesSpawned"], 2);
    let contract: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract")).expect("JSON");
    let extra = mcp_call(
        root_path,
        &candidate_runtime(),
        "work_item_amend",
        json!({"workItemId":wi,"request":{
            "schemaVersion":1,"changeId":"retry-preapproved-command","expectedContractDigest":cockpit_protocol::digest_json(&contract).expect("digest"),
            "reason":"exercise a genuine failed preapproved check and bounded retry",
            "changes":[{"path":"/verification","operation":"add","value":"git ls-files --error-unmatch retry-fixture.txt"}],
            "developmentReview":{"actor":"agent:fixture","authoritySource":"user:synthetic-flow","assurance":"self_declared","operations":["review_development_amendment"],"policyRefs":[],"evidenceRefs":[]}
        }}),
    );
    assert_eq!(
        extra.pointer("/result/isError"),
        Some(&json!(false)),
        "{extra}"
    );
    assert_success(
        &run_cli(&[
            "preflight",
            "--repo",
            root_path.to_str().expect("path"),
            "--contract",
            contract_path.to_str().expect("path"),
        ]),
        "retry-check preflight",
    );
    let failed = run_cli(&[
        "verify",
        "--repo",
        root_path.to_str().expect("path"),
        "--work-item",
        wi,
    ]);
    assert!(
        !failed.status.success(),
        "missing tracked fixture must produce a real command failure"
    );
    let evidence = root_path.join(".ai/evidence");
    let failed_files: Vec<_> = fs::read_dir(&evidence)
        .expect("evidence")
        .map(|e| e.expect("entry").path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(&format!("{wi}.verification-attempt.")))
        })
        .filter_map(|p| {
            let bytes = fs::read(&p).expect("attempt");
            let value: Value = serde_json::from_slice(&bytes).expect("JSON");
            (value["state"] == "execution_failed").then_some((p, bytes))
        })
        .collect();
    assert_eq!(failed_files.len(), 1);
    fs::write(
        root_path.join("retry-fixture.txt"),
        "satisfy the already approved check\n",
    )
    .expect("in-scope correction");
    git(root_path, &["add", "retry-fixture.txt"]);
    git(root_path, &["commit", "-qm", "correct the scoped fixture"]);
    let active = root_path.join(".ai/work-items/active");
    let c: Value =
        serde_json::from_slice(&fs::read(&contract_path).expect("Contract")).expect("JSON");
    let summary: Value = serde_json::from_slice(
        &fs::read(active.join(format!("{wi}.summary.json"))).expect("Summary"),
    )
    .expect("JSON");
    let runtime = candidate_runtime();
    let mut retry = ::serde_json::json!({
        "schemaVersion":1,"decisionId":"work-item-recovery","decision":"retry","workItemId":wi,"repositoryId":repository_id(root_path),
        "predecessorWorkItemId":wi,"predecessorContractDigest":cockpit_protocol::digest_json(&c).expect("digest"),
        "predecessorSummaryDigest":cockpit_protocol::digest_json(&summary).expect("digest"),
        "runtimeVersion":runtime.runtime_version,"runtimeDigest":runtime.runtime_digest,"actor":"agent:fixture",
        "authoritySource":"user:synthetic-flow","reason":"retry the same exact approved checks after scoped correction; not a human review",
        "decidedAt":"2026-10-23T00:00:00Z","resumeCondition":"execute all exact current checks"
    });
    let outcome_path = active.join(format!("{wi}.outcome.json"));
    if outcome_path.exists() {
        let outcome: Value =
            serde_json::from_slice(&fs::read(outcome_path).expect("Outcome")).expect("JSON");
        retry["predecessorOutcomeDigest"] =
            json!(cockpit_protocol::digest_json(&outcome).expect("digest"));
    }
    let events_path = active.join(format!("{wi}.events.jsonl"));
    if events_path.exists() {
        retry["predecessorEventsDigest"] = json!(Digest::sha256_bytes(
            &fs::read(events_path).expect("events")
        ));
    }
    record_recovery_decision(root_path, wi, &retry, &runtime)
        .expect("actual bounded retry receipt");
    assert_success(
        &run_cli(&[
            "verify",
            "--repo",
            root_path.to_str().expect("path"),
            "--work-item",
            wi,
        ]),
        "fresh retry",
    );
    let envelope: Value = serde_json::from_slice(
        &fs::read(evidence.join(format!("{wi}.verification.json"))).expect("verification"),
    )
    .expect("JSON");
    assert_eq!(envelope["receipt"]["nodesExecuted"], 3);
    assert_eq!(envelope["receipt"]["nodesReused"], 0);
    for (path, bytes) in failed_files {
        assert_eq!(fs::read(path).expect("old failure remains"), bytes);
    }
    assert_eq!(fs::read(owner_path).expect("owner unchanged"), owner_bytes);
}

#[test]
fn initial_verified_or_malformed_scenarios_do_not_create_lifecycle_records() {
    for (i,entry) in [json!({"scenario":"already-passed","required":true,"status":"verified","evidence":["invented"]}),
        json!({"scenario":"","required":true,"status":"unverified","evidence":[]})].into_iter().enumerate() {
        let root=repository();let wi=format!("WI-INVALID-INITIAL-{i}");let input=tempfile::NamedTempFile::new().expect("input");
        fs::write(input.path(),serde_json::to_vec(&json!([entry.clone()])).expect("JSON")).expect("input");
        let output=run_cli(&["start","--repo",root.path().to_str().expect("path"),"--id",&wi,"--intent","initial plan","--goal","reject invalid plan","--scope","**","--authority","authorized","--scenario-coverage",input.path().to_str().expect("path")]);
        assert!(!output.status.success());assert!(!root.path().join(format!(".ai/work-items/active/{wi}.contract.json")).exists());
        let mcp=mcp_call(root.path(),&candidate_runtime(),"work_item_start",json!({"workItemId":wi,"intent":"initial plan","goal":"reject invalid plan","scope":["**"],"authority":"authorized","scenarioCoverage":[entry]}));
        assert_eq!(mcp.pointer("/result/isError"),Some(&json!(true)),"{mcp}");
        assert!(!root.path().join(format!(".ai/work-items/active/{wi}.summary.json")).exists());
    }
}

#[test]
fn initial_unverified_scenario_with_evidence_is_rejected_before_record_creation() {
    for transport in ["cli", "mcp"] {
        let root = repository();
        let root_path = root.path();
        let wi = "WI-INITIAL-EVIDENCE-CONTROL";
        let active = root_path.join(".ai/work-items/active");
        for reject in [true, false] {
            let evidence = if reject {
                json!(["invented"])
            } else {
                json!([])
            };
            let scenarios = json!([{
                "scenario": "initial-evidence-boundary",
                "required": true,
                "status": "unverified",
                "evidence": evidence,
                "expected": "Only an empty-evidence initial plan is accepted.",
                "verificationPlan": "git diff --check",
                "reason": null,
                "expectedResult": null,
                "expectedOutcome": null,
                "description": null
            }]);
            let diagnostic = "initial plan must be unverified with empty evidence";
            if transport == "cli" {
                let input = tempfile::NamedTempFile::new().expect("scenario input");
                fs::write(input.path(), serde_json::to_vec(&scenarios).expect("JSON"))
                    .expect("write scenario input");
                let output = run_cli(&[
                    "start",
                    "--repo",
                    root_path.to_str().expect("repository path"),
                    "--id",
                    wi,
                    "--intent",
                    "initial evidence boundary",
                    "--goal",
                    "accept an otherwise identical empty-evidence plan",
                    "--scope",
                    "**",
                    "--authority",
                    "authorized",
                    "--acceptance",
                    "initial execution evidence is rejected",
                    "--verification",
                    "git diff --check",
                    "--scenario-coverage",
                    input.path().to_str().expect("input path"),
                ]);
                if reject {
                    assert!(!output.status.success(), "evidence-bearing CLI start");
                    assert!(
                        String::from_utf8_lossy(&output.stderr).contains(diagnostic),
                        "wrong CLI rejection: {output:?}"
                    );
                } else {
                    assert_success(&output, "empty-evidence CLI control");
                }
            } else {
                let response = mcp_call(
                    root_path,
                    &candidate_runtime(),
                    "work_item_start",
                    json!({
                        "workItemId": wi,
                        "intent": "initial evidence boundary",
                        "goal": "accept an otherwise identical empty-evidence plan",
                        "scope": ["**"],
                        "authority": "authorized",
                        "acceptanceCriteria": ["initial execution evidence is rejected"],
                        "verification": ["git diff --check"],
                        "scenarioCoverage": scenarios
                    }),
                );
                assert_eq!(
                    response.pointer("/result/isError"),
                    Some(&json!(reject)),
                    "{response}"
                );
                if reject {
                    assert!(response.to_string().contains(diagnostic), "{response}");
                }
            }
            if reject {
                for suffix in [
                    "contract.json",
                    "summary.json",
                    "events.jsonl",
                    "outcome.json",
                ] {
                    assert!(
                        !active.join(format!("{wi}.{suffix}")).exists(),
                        "{transport}: rejected input created {suffix}"
                    );
                }
            } else {
                for suffix in ["contract", "summary"] {
                    let saved: Value = serde_json::from_slice(
                        &fs::read(active.join(format!("{wi}.{suffix}.json")))
                            .expect("accepted control record"),
                    )
                    .expect("record JSON");
                    assert_eq!(
                        saved["scenarioCoverage"], scenarios,
                        "{transport}: {suffix}"
                    );
                }
            }
        }
    }
}

#[test]
fn explicit_task_caller_and_resumption_assessment_match_cli_and_mcp() {
    let root = repository();
    let mut request = typed_request(root.path());
    request.change_id = "bounded-task-adapters".into();
    request.changes = vec![ContractAmendmentChange {
        path: "/governanceProfile".into(),
        operation: ContractAmendmentOperation::Set,
        value: Some(json!({"taskAuthorization":{
            "schemaVersion":1,"agentActor":"agent:adapter-task", "authoritySource":"fixture:explicit-owner",
            "allowedActions":["modify_source","run_preflight","run_verification","record_governance_controls"],
            "closeoutActions":[],"expiresAt":"2099-01-01T00:00:00Z","publicTarget":null
        }})),
    }];
    cockpit_repository::apply_work_item_contract_amendment(
        root.path(),
        WORK_ITEM_ID,
        &request,
        &candidate_runtime(),
    )
    .unwrap();
    let repository_path = root.path().to_str().unwrap();
    let contract_path = root.path().join(format!(
        ".ai/work-items/active/{WORK_ITEM_ID}.contract.json"
    ));
    let preflight = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        contract_path.to_str().unwrap(),
    ]);
    assert_success(&preflight, "native owner-review observation");
    let contract = retained_document(root.path(), "contract");
    let summary = retained_document(root.path(), "summary");
    record_work_item_governance_controls_with_runtime(
        root.path(),
        WORK_ITEM_ID,
        &json!({"decisionEvidence":{
            "schemaVersion":1,"decisionId":"contract-preflight-review","decision":"confirm_review",
            "repositoryId":repository_id(root.path()),"workItemId":WORK_ITEM_ID,
            "contractDigest":cockpit_protocol::digest_json(&contract).unwrap(),
            "preflightDecisionDigest":summary["preflightDecisionDigest"],
            "repositorySnapshotDigest":summary["preflightRepositorySnapshotDigest"],
            "recordedAt":"2026-10-24T18:00:00Z","recordedBy":"agent:isolated-test-recorder",
            "reason":"Synthetic native adapter fixture, not external human identity authentication."
        }}),
        &candidate_runtime(),
    )
    .unwrap();
    let caller = json!({"schemaVersion":1,"actor":"agent:adapter-task","authoritySource":"fixture:explicit-owner","assurance":"self_declared"});
    let query = json!({"schemaVersion":1,"caller":caller,"action":"modify_source","paths":["README.md"],"publicTarget":null});
    let input = tempfile::NamedTempFile::new().unwrap();
    fs::write(input.path(), serde_json::to_vec(&query).unwrap()).unwrap();
    let output = run_cli(&[
        "work-item",
        "status",
        "--repo",
        repository_path,
        "--id",
        WORK_ITEM_ID,
        "--json",
        "--authorization-request",
        input.path().to_str().unwrap(),
    ]);
    assert_success(&output, "CLI exact task intent");
    let cli: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mcp = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_status",
        json!({"workItemId":WORK_ITEM_ID,"authorizationRequest":query}),
    );
    assert_eq!(mcp["result"]["isError"], false, "{mcp}");
    assert_eq!(
        cli["authorizationAssessment"],
        mcp["result"]["structuredContent"]["authorizationAssessment"]
    );
    assert_eq!(cli["authorizationAssessment"]["authorized"], true);
    let all = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_status",
        json!({"all":true,"authorizationRequest":query}),
    );
    assert_eq!(
        all["result"]["isError"], true,
        "a single task grant cannot authorize all WIs: {all}"
    );
    let missing = run_cli(&[
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        contract_path.to_str().unwrap(),
    ]);
    assert!(
        !missing.status.success(),
        "legacy call must not infer the configured agent"
    );
    let context_file = tempfile::NamedTempFile::new().unwrap();
    fs::write(context_file.path(), serde_json::to_vec(&caller).unwrap()).unwrap();
    assert_success(
        &run_cli(&[
            "--task-caller",
            context_file.path().to_str().unwrap(),
            "preflight",
            "--repo",
            repository_path,
            "--contract",
            contract_path.to_str().unwrap(),
        ]),
        "explicit CLI caller",
    );
    let mcp_preflight = mcp_call(
        root.path(),
        &candidate_runtime(),
        "preflight",
        json!({"contract":format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json"),"taskCaller":caller}),
    );
    assert_eq!(mcp_preflight["result"]["isError"], false, "{mcp_preflight}");
    let grant = cli["authorizationAssessment"]["authority"]["grantId"].clone();
    let narrowed = mcp_call(
        root.path(),
        &candidate_runtime(),
        "work_item_controls",
        json!({"workItemId":WORK_ITEM_ID,"input":{"taskAuthorizationTermination":{
        "schemaVersion":1,"grantId":grant,"decision":"withdrawn","caller":caller,"reason":"Explicit synthetic agent withdraws the exact local grant."
    }},"taskCaller":caller}),
    );
    assert_eq!(narrowed["result"]["isError"], false, "{narrowed}");
    let after = run_cli(&[
        "--task-caller",
        context_file.path().to_str().unwrap(),
        "preflight",
        "--repo",
        repository_path,
        "--contract",
        contract_path.to_str().unwrap(),
    ]);
    assert!(!after.status.success());
    assert!(String::from_utf8_lossy(&after.stderr).contains("withdrawn"));
    let rejected = mcp_call(
        root.path(),
        &candidate_runtime(),
        "preflight",
        json!({"contract":format!(".ai/work-items/active/{WORK_ITEM_ID}.contract.json"),"taskCaller":caller}),
    );
    assert_eq!(rejected["result"]["isError"], true, "{rejected}");
}

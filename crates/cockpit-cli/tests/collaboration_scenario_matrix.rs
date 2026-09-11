//! Minimal production-path acceptance for the collaboration scenario matrix.
//!
//! The fixture is created in a temporary repository through the CLI lifecycle
//! commands. Outcome facts are read from the real CLI subprocess and the real
//! MCP stdio server; this test never constructs an OutcomeV2 or calls the
//! renderer directly.

use std::{
    fs,
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

use sha2::{Digest as ShaDigest, Sha256};

mod common;

fn repository() -> tempfile::TempDir {
    let directory = tempfile::tempdir().expect("tempdir");
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .current_dir(directory.path())
            .status()
            .expect("git init")
            .success()
    );
    directory
}

fn run_json(binary: &str, repo: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::new(binary)
        .args(args)
        .args(["--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("run ai-cockpit");
    assert!(
        output.status.success(),
        "args={args:?}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("machine-readable stdout JSON")
}

fn cli_outcome(binary: &str, repo: &Path, id: &str) -> serde_json::Value {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--id", id, "--json", "--repo"])
        .arg(repo)
        .current_dir(repo)
        .output()
        .expect("CLI outcome");
    assert!(
        output.status.success(),
        "CLI outcome stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI Outcome JSON")
}

fn cli_handoff(binary: &str, repo: &Path, id: &str, language: &str, view: &str) -> String {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--id", id, "--view", view, "--repo"])
        .arg(repo)
        .env("AI_COCKPIT_LANGUAGE", language)
        .current_dir(repo)
        .output()
        .expect("CLI human outcome");
    assert!(
        output.status.success(),
        "language={language}, view={view}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("CLI human outcome UTF-8")
}

fn mcp_stdio_response(
    binary: &str,
    repo: &Path,
    id: &str,
    language: &str,
    view: &str,
) -> serde_json::Value {
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "tools/call",
        "params": {
            "name": "work_item_outcome",
            "arguments": {
                "workItemId": id,
                "language": language,
                "view": view
            }
        }
    });
    let mut child = Command::new(binary)
        .args(["mcp", "--repo"])
        .arg(repo)
        .current_dir(repo)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("MCP stdio server");
    let mut request_bytes = serde_json::to_vec(&request).expect("MCP request JSON");
    request_bytes.push(b'\n');
    child
        .stdin
        .as_mut()
        .expect("MCP stdin")
        .write_all(&request_bytes)
        .expect("MCP request");
    let output = child.wait_with_output().expect("MCP response");
    assert!(
        output.status.success(),
        "MCP stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("MCP JSON-RPC response")
}

fn expected_language(language: &str) -> &str {
    match language {
        "zh-CN" => "zh",
        "ja" => "ja",
        _ => "en",
    }
}

fn normalized_handoff(text: &str) -> &str {
    text.trim_end_matches(['\r', '\n'])
}

fn expected_headings(language: &str, view: &str) -> Vec<&'static str> {
    match (expected_language(language), view) {
        ("en", "summary") => vec![
            "Result",
            "Key changes",
            "Remaining uncertainty",
            "Human next step",
        ],
        ("zh", "summary") => vec!["结果", "关键变化", "剩余不确定性", "人的下一步"],
        ("ja", "summary") => vec!["結果", "主な変更", "残る不確実性", "人間の次のアクション"],
        ("en", "full") => vec![
            "Task Result",
            "What was completed",
            "Problems found",
            "Stops triggered",
            "Unknowns",
            "Human decisions",
            "Verification",
            "Impact",
            "Next action",
            "Evidence",
        ],
        ("zh", "full") => vec![
            "结果",
            "已完成",
            "发现的问题",
            "触发的停止",
            "未知项",
            "人工决定",
            "验证",
            "影响",
            "下一步",
            "证据",
        ],
        ("ja", "full") => vec![
            "結果",
            "完了したこと",
            "発見された問題",
            "発動した停止",
            "不明点",
            "人間の判断",
            "検証",
            "影響",
            "次のアクション",
            "証拠",
        ],
        (_, _) => unreachable!("unsupported view: {view}"),
    }
}

fn required_fragments(language: &str) -> [&'static str; 2] {
    match expected_language(language) {
        "zh" => ["不是验证证据无效", "意图对齐证据不足"],
        "ja" => [
            "検証 evidence が無効という意味ではありません",
            "intent alignment evidence が不足しています",
        ],
        _ => [
            "does not mean verification evidence is invalid",
            "intent-alignment evidence is insufficient",
        ],
    }
}

fn assert_outcome_semantics_equal(
    cli_outcome: &serde_json::Value,
    mcp_outcome: &serde_json::Value,
    work_item_id: &str,
) {
    assert_eq!(cli_outcome, mcp_outcome, "CLI/MCP Outcome objects diverged");
    assert_eq!(cli_outcome["workItemId"], work_item_id);
    for field in [
        "state",
        "decisionState",
        "unknowns",
        "governanceReasons",
        "finalization",
    ] {
        assert_eq!(
            cli_outcome[field], mcp_outcome[field],
            "CLI/MCP semantic field diverged: {field}"
        );
    }
    assert!(
        cli_outcome["unknowns"]
            .as_array()
            .is_some_and(|unknowns| !unknowns.is_empty()),
        "fixture must expose at least one explicit unknown"
    );
    assert!(
        cli_outcome["governanceReasons"]
            .as_array()
            .is_some_and(|reasons| {
                reasons
                    .iter()
                    .any(|reason| reason == "acceptance_evidence_insufficient")
                    && reasons
                        .iter()
                        .any(|reason| reason == "intent_alignment_insufficient")
            }),
        "fixture must expose the expected governance reasons: {cli_outcome}"
    );
    assert_eq!(cli_outcome["finalization"]["state"], "receipt_missing");
    assert_eq!(
        cli_outcome["finalization"]["action"],
        "record_finalization_receipt"
    );
    assert_eq!(
        cli_outcome["finalization"]["nextAction"]["id"],
        "record_finalization_receipt"
    );
}

#[test]
fn collaboration_matrix_fixture_preserves_outcome_semantics_through_cli_and_mcp_stdio() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let work_item_id = "WI-COLLABORATION-MATRIX";
    let repository = repository();

    // Reuse the established real lifecycle fixture: the Work Item is
    // verified, but acceptance/intent controls remain incomplete and the
    // finalization receipt is absent.
    run_json(binary, repository.path(), &["attach"]);
    run_json(
        binary,
        repository.path(),
        &[
            "start",
            "--id",
            work_item_id,
            "--intent",
            "prove CLI and MCP preserve one repository-bound Outcome",
            "--goal",
            "exercise summary/full and en/zh-CN/ja through production entrypoints",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: compare status, unknowns, reasons, actions, and human handoff",
            "--required-evidence",
            "verification",
        ],
    );
    common::plan(binary, repository.path(), work_item_id);
    run_json(
        binary,
        repository.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{work_item_id}.contract.json"),
        ],
    );
    run_json(
        binary,
        repository.path(),
        &["checkpoint", "--id", work_item_id],
    );
    run_json(
        binary,
        repository.path(),
        &["verify", "--work-item", work_item_id, "--command", "true"],
    );

    let cli_outcome = cli_outcome(binary, repository.path(), work_item_id);
    for language in ["en", "zh-CN", "ja"] {
        for view in ["summary", "full"] {
            let cli_handoff = cli_handoff(binary, repository.path(), work_item_id, language, view);
            let mcp_response =
                mcp_stdio_response(binary, repository.path(), work_item_id, language, view);
            assert_eq!(mcp_response["result"]["isError"], false);
            let structured = &mcp_response["result"]["structuredContent"];
            let mcp_outcome = &structured["outcome"];
            assert_outcome_semantics_equal(&cli_outcome, mcp_outcome, work_item_id);
            assert_eq!(structured["language"], expected_language(language));

            let mcp_handoff = structured["humanHandoff"]
                .as_str()
                .expect("MCP structured human handoff");
            assert_eq!(
                mcp_response["result"]["content"][0]["text"], mcp_handoff,
                "MCP content and structured human handoff diverged"
            );
            assert_eq!(
                normalized_handoff(&cli_handoff),
                normalized_handoff(mcp_handoff),
                "CLI and MCP stdio human handoffs diverged for language={language}, view={view}"
            );
            assert!(cli_handoff.starts_with("Outcome:"));
            for heading in expected_headings(language, view) {
                assert!(
                    cli_handoff.contains(heading),
                    "language={language}, view={view}, missing heading {heading:?}: {cli_handoff}"
                );
            }
            for fragment in required_fragments(language) {
                assert!(
                    cli_handoff.contains(fragment),
                    "language={language}, view={view}, missing semantic fragment {fragment:?}: {cli_handoff}"
                );
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum FinalizationCase {
    Retained,
    Deleted,
    Abandoned,
    Corrupt,
    IdentityMismatch,
    CleanupPending,
}

impl FinalizationCase {
    fn slug(self) -> &'static str {
        match self {
            Self::Retained => "RETAINED",
            Self::Deleted => "DELETED",
            Self::Abandoned => "ABANDONED",
            Self::Corrupt => "CORRUPT",
            Self::IdentityMismatch => "IDENTITY-MISMATCH",
            Self::CleanupPending => "CLEANUP-PENDING",
        }
    }

    fn expected_observation_state(self) -> &'static str {
        match self {
            Self::Retained => "verified_retained",
            Self::Deleted => "verified_deleted",
            Self::Abandoned => "verified_abandoned",
            Self::Corrupt => "record_corrupt",
            Self::IdentityMismatch => "identity_mismatch",
            Self::CleanupPending => "cleanup_pending",
        }
    }

    fn expected_next_action(self) -> &'static str {
        match self {
            Self::Retained => "preserve_historical_evidence",
            Self::Deleted | Self::Abandoned => "record_close_decision",
            Self::Corrupt | Self::IdentityMismatch => "inspect_recovery_conditions",
            Self::CleanupPending => "verify_finalization_receipt",
        }
    }

    fn disposition(self) -> &'static str {
        match self {
            Self::Retained => "retained",
            Self::Deleted | Self::CleanupPending | Self::IdentityMismatch => "deleted",
            Self::Abandoned => "abandoned",
            Self::Corrupt => "deleted",
        }
    }
}

struct FinalizationFixture {
    repository: tempfile::TempDir,
    work_item_id: String,
}

fn git_text(repo: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("git output UTF-8")
        .trim()
        .to_owned()
}

fn git_success(repo: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git command");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn plan_with_context(binary: &str, repo: &Path, work_item_id: &str, context: &serde_json::Value) {
    let context_file = tempfile::NamedTempFile::new().expect("resource context file");
    fs::write(
        context_file.path(),
        serde_json::to_vec_pretty(context).expect("resource context JSON"),
    )
    .expect("write resource context");
    let context_path = context_file.path().to_string_lossy().into_owned();
    run_json(
        binary,
        repo,
        &[
            "work-item",
            "finalize-plan",
            "--id",
            work_item_id,
            "--input",
            &context_path,
        ],
    );
}

fn runtime_digest(binary: &str) -> String {
    let bytes = fs::read(binary).expect("runtime binary");
    format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
}

fn finalization_receipt(
    binary: &str,
    repo: &Path,
    work_item_id: &str,
    context: &serde_json::Value,
    case: FinalizationCase,
) -> serde_json::Value {
    let status = run_json(binary, repo, &["status"]);
    let repository_id = status["repositoryId"]
        .as_str()
        .expect("repository identity")
        .to_owned();
    let contract_path = repo
        .join(".ai/work-items/archive")
        .join(format!("{work_item_id}.contract.json"));
    let contract_bytes = fs::read(&contract_path).expect("archived Contract");
    let contract: serde_json::Value =
        serde_json::from_slice(&contract_bytes).expect("archived Contract JSON");
    let branch = context["branch"].as_str().expect("context branch");
    let worktree = context["worktree"].as_str().expect("context worktree");
    let base_branch = context["baseBranch"].as_str().expect("context base branch");
    let base_remote = context["baseRemote"].as_str().expect("context base remote");
    let pull_request = context["pullRequest"].as_str().expect("context PR");
    let head_revision = format!("head-{}", case.slug().to_lowercase());
    let abandoned = matches!(case, FinalizationCase::Abandoned);
    let receipt = serde_json::json!({
        "schemaVersion": 1,
        "receiptId": format!("receipt-{work_item_id}-{}", case.slug()),
        "operationId": format!("operation-{work_item_id}-{}", case.slug()),
        "repositoryId": repository_id,
        "workItemId": work_item_id,
        "runtimeVersion": env!("CARGO_PKG_VERSION"),
        "runtimeDigest": runtime_digest(binary),
        "provider": context["provider"],
        "pullRequest": {
            "number": 1,
            "url": pull_request,
            "headRevision": head_revision,
            "baseBranch": base_branch,
            "baseRemote": base_remote,
            "baseRevision": contract["baseRevision"],
            "mergeCommit": if abandoned { serde_json::Value::Null } else { serde_json::json!("merge-test") }
        },
        "branch": {
            "name": branch,
            "remote": base_remote,
            "headRevision": format!("head-{}", case.slug().to_lowercase())
        },
        "worktree": {
            "worktreeId": format!("wt-{work_item_id}"),
            "path": worktree,
            "branch": branch,
            "headRevision": format!("head-{}", case.slug().to_lowercase())
        },
        "before": {
            "pullRequest": if abandoned { "unmerged" } else { "merged" },
            "branch": "present",
            "worktree": "clean"
        },
        "after": {
            "pullRequest": if abandoned { "unmerged" } else { "merged" },
            "branch": if matches!(case, FinalizationCase::Retained) { "present" } else { "deleted" },
            "worktree": if matches!(case, FinalizationCase::Retained) { "clean" } else { "removed" }
        },
        "result": {
            "disposition": case.disposition(),
            "failureCodes": if abandoned { vec!["unmerged_pull_request"] } else { Vec::<&str>::new() },
            "unknownCodes": []
        },
        "actor": "human:test",
        "authoritySource": "test-policy",
        "reason": "controlled collaboration matrix fixture",
        "timestamp": "2026-08-23T00:00:00Z",
        "contractDigest": format!("sha256:{}", hex::encode(Sha256::digest(&contract_bytes))),
        "resourceContext": context
    });
    if matches!(case, FinalizationCase::IdentityMismatch) {
        let mut mismatched = receipt.clone();
        mismatched["repositoryId"] = format!("sha256:{}", "b".repeat(64)).into();
        return mismatched;
    }
    receipt
}

fn finalization_fixture(binary: &str, case: FinalizationCase) -> FinalizationFixture {
    let repository = repository();
    let root = repository.path();
    let work_item_id = format!("WI-COLLAB-FINALIZATION-{}", case.slug());
    fs::write(
        root.join("README.md"),
        "collaboration finalization fixture\n",
    )
    .expect("fixture README");
    git_success(root, &["config", "user.email", "tests@example.invalid"]);
    git_success(root, &["config", "user.name", "AI Cockpit Tests"]);
    git_success(root, &["add", "README.md"]);
    git_success(root, &["commit", "-q", "-m", "fixture base"]);
    run_json(binary, root, &["attach"]);
    run_json(
        binary,
        root,
        &[
            "start",
            "--id",
            &work_item_id,
            "--intent",
            "exercise a real CLI and MCP finalization observation",
            "--goal",
            "keep typed next_action stable across views and locales",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
        ],
    );

    let current_branch = git_text(root, &["rev-parse", "--abbrev-ref", "HEAD"]);
    let cleanup_pending = matches!(case, FinalizationCase::CleanupPending);
    let context = if cleanup_pending {
        serde_json::json!({
            "branch": current_branch,
            "worktree": root.to_string_lossy(),
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/798"
        })
    } else {
        serde_json::json!({
            "branch": format!("feature/{work_item_id}"),
            "worktree": format!("/tmp/removed-{work_item_id}"),
            "baseBranch": "main",
            "baseRemote": "origin",
            "provider": "github",
            "pullRequest": "https://github.com/example/ai-cockpit/pull/798"
        })
    };
    plan_with_context(binary, root, &work_item_id, &context);
    run_json(
        binary,
        root,
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{work_item_id}.contract.json"),
        ],
    );
    run_json(binary, root, &["checkpoint", "--id", &work_item_id]);
    run_json(
        binary,
        root,
        &["verify", "--work-item", &work_item_id, "--command", "true"],
    );
    run_json(binary, root, &["finish", "--id", &work_item_id]);
    run_json(binary, root, &["archive", "--id", &work_item_id]);

    let receipt = finalization_receipt(binary, root, &work_item_id, &context, case);
    let receipt_path = root
        .join(".ai/decisions")
        .join(format!("{work_item_id}.finalize.json"));
    if matches!(case, FinalizationCase::Corrupt) {
        fs::write(receipt_path, b"{ this is not a finalization receipt").expect("corrupt fixture");
    } else if matches!(
        case,
        FinalizationCase::IdentityMismatch | FinalizationCase::CleanupPending
    ) {
        fs::write(
            receipt_path,
            serde_json::to_vec_pretty(&receipt).expect("non-terminal receipt JSON"),
        )
        .expect("write non-terminal finalization fixture");
    } else {
        let receipt_file = tempfile::NamedTempFile::new().expect("receipt fixture");
        fs::write(
            receipt_file.path(),
            serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
        )
        .expect("write receipt fixture");
        run_json(
            binary,
            root,
            &[
                "work-item",
                "finalize",
                "--id",
                &work_item_id,
                "--input",
                &receipt_file.path().to_string_lossy(),
            ],
        );
    }
    FinalizationFixture {
        repository,
        work_item_id,
    }
}

fn cli_machine_outcome(
    binary: &str,
    repo: &Path,
    work_item_id: &str,
    language: &str,
    view: &str,
) -> serde_json::Value {
    let output = Command::new(binary)
        .args([
            "work-item",
            "outcome",
            "--id",
            work_item_id,
            "--json",
            "--view",
            view,
            "--repo",
        ])
        .arg(repo)
        .env("AI_COCKPIT_LANGUAGE", language)
        .current_dir(repo)
        .output()
        .expect("CLI machine outcome");
    assert!(
        output.status.success(),
        "language={language}, view={view}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("CLI machine Outcome JSON")
}

fn assert_typed_finalization(
    outcome: &serde_json::Value,
    work_item_id: &str,
    case: FinalizationCase,
) {
    assert_eq!(outcome["workItemId"], work_item_id);
    let finalization = &outcome["finalization"];
    assert_eq!(
        finalization["observationState"],
        case.expected_observation_state(),
        "unexpected typed finalization state for {case:?}: {outcome}"
    );
    assert_eq!(
        finalization["nextAction"]["id"],
        case.expected_next_action(),
        "unexpected typed next_action for {case:?}: {outcome}"
    );
    assert!(
        finalization["nextAction"]["authorization"].is_string(),
        "typed next_action must retain authorization: {outcome}"
    );
    assert!(
        finalization["nextAction"]["safety"].is_string(),
        "typed next_action must retain safety: {outcome}"
    );
}

#[test]
fn finalization_cases_keep_typed_next_action_equal_across_cli_mcp_views_and_locales() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    for case in [
        FinalizationCase::Retained,
        FinalizationCase::Deleted,
        FinalizationCase::Abandoned,
        FinalizationCase::Corrupt,
        FinalizationCase::IdentityMismatch,
        FinalizationCase::CleanupPending,
    ] {
        let fixture = finalization_fixture(binary, case);
        let root = fixture.repository.path();
        let mut baseline: Option<serde_json::Value> = None;
        for language in ["en", "zh-CN", "ja"] {
            for view in ["summary", "full"] {
                let cli_outcome =
                    cli_machine_outcome(binary, root, &fixture.work_item_id, language, view);
                assert_typed_finalization(&cli_outcome, &fixture.work_item_id, case);
                if let Some(expected) = &baseline {
                    assert_eq!(
                        &cli_outcome["finalization"]["nextAction"],
                        &expected["finalization"]["nextAction"],
                        "CLI typed next_action changed for {case:?}, language={language}, view={view}"
                    );
                    assert_eq!(
                        &cli_outcome["finalization"]["observationState"],
                        &expected["finalization"]["observationState"],
                        "CLI typed observation state changed for {case:?}, language={language}, view={view}"
                    );
                } else {
                    baseline = Some(cli_outcome.clone());
                }

                let mcp_response =
                    mcp_stdio_response(binary, root, &fixture.work_item_id, language, view);
                assert_eq!(mcp_response["result"]["isError"], false);
                let structured = &mcp_response["result"]["structuredContent"];
                let mcp_outcome = &structured["outcome"];
                assert_typed_finalization(mcp_outcome, &fixture.work_item_id, case);
                assert_eq!(
                    &cli_outcome["finalization"]["nextAction"],
                    &mcp_outcome["finalization"]["nextAction"],
                    "CLI/MCP typed next_action diverged for {case:?}, language={language}, view={view}"
                );
                assert_eq!(
                    &cli_outcome["finalization"]["observationState"],
                    &mcp_outcome["finalization"]["observationState"],
                    "CLI/MCP typed observation state diverged for {case:?}, language={language}, view={view}"
                );

                let cli_handoff = cli_handoff(binary, root, &fixture.work_item_id, language, view);
                let mcp_handoff = structured["humanHandoff"]
                    .as_str()
                    .expect("MCP human handoff");
                assert_eq!(
                    normalized_handoff(&cli_handoff),
                    normalized_handoff(mcp_handoff),
                    "CLI/MCP handoff diverged for {case:?}, language={language}, view={view}"
                );
            }
        }
    }
}

#[test]
fn unknown_finalization_case_is_explicitly_unavailable_in_the_matrix() {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/reference/collaboration-scenario-matrix.json"
    ))
    .expect("scenario matrix JSON");
    let scenario = matrix["scenarios"]
        .as_array()
        .expect("scenario list")
        .iter()
        .find(|scenario| scenario["id"] == "SCN-033")
        .expect("unknown finalization scenario");
    assert_eq!(scenario["sourceType"], "unavailable");
    assert_eq!(scenario["expected"]["result"], "unavailable");
    assert!(
        scenario["notes"]
            .as_str()
            .is_some_and(|notes| notes.contains("cannot be safely constructed")),
        "unknown case must retain explicit unavailable evidence: {scenario}"
    );
}

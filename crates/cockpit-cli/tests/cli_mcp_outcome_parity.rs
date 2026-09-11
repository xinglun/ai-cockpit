use sha2::{Digest as ShaDigest, Sha256};
use std::{collections::BTreeSet, fs, path::Path, process::Command};

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

/// Reconstruct the exact `RuntimeContext` that the `binary` under test would
/// compute for itself (see `crates/cockpit-cli/src/runtime_identity.rs`), so
/// an MCP call made in-process against the same repository is bound to the
/// identical runtime identity as the CLI subprocess, and the two outputs can
/// be compared for genuine equality rather than differing only on
/// incidental runtime metadata.
fn current_runtime_context(binary: &str) -> cockpit_protocol::RuntimeContext {
    let bytes = fs::read(binary).expect("read runtime binary");
    let digest = format!("sha256:{}", hex::encode(Sha256::digest(bytes)))
        .parse::<cockpit_core::Digest>()
        .expect("valid digest");
    cockpit_protocol::RuntimeContext {
        runtime_version: env!("CARGO_PKG_VERSION").into(),
        protocol_version: cockpit_protocol::PROTOCOL_VERSION,
        runtime_digest: digest,
    }
}

fn human_cli_output(binary: &str, repo: &Path, id: &str, language: &str) -> String {
    let output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo)
        .args(["--id", id])
        .env("AI_COCKPIT_LANGUAGE", language)
        .output()
        .expect("cli human outcome");
    assert!(
        output.status.success(),
        "language={language}, stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("human CLI UTF-8")
}

fn human_semantic_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            line.starts_with("Outcome:")
                || line.starts_with("- Verification:")
                || line.starts_with("- 验证状态:")
                || line.starts_with("- 検証状態:")
                || line.starts_with("- Lifecycle:")
                || line.starts_with("- 生命周期状态:")
                || line.starts_with("- ライフサイクル状態:")
                || line.starts_with("- Human decision:")
                || line.starts_with("- 人工决定状态:")
                || line.starts_with("- 人間の判断状態:")
                || line.starts_with("- Governance signal:")
                || line.starts_with("- 治理信号:")
                || line.starts_with("- ガバナンスシグナル:")
                || line.starts_with("Human next step")
                || line.starts_with("人的下一步")
                || line.starts_with("人間の次のアクション")
        })
        .collect()
}

fn human_semantic_tokens(text: &str, language: &str) -> BTreeSet<&'static str> {
    let mut tokens = BTreeSet::new();
    let contains_any = |needles: &[&str]| needles.iter().any(|needle| text.contains(needle));
    if text.starts_with("Outcome:") {
        tokens.insert("outcome_marker");
    }
    if contains_any(&["Verification:", "验证状态:", "検証状態:"]) {
        tokens.insert("verification_state");
    }
    if contains_any(&[
        "acceptance evidence",
        "受入れ evidence",
        "受入 evidence",
        "验收证据",
    ]) {
        tokens.insert("acceptance_gap");
    }
    if contains_any(&[
        "intent alignment",
        "intent-alignment",
        "意図",
        "意図の整合",
        "意图对齐",
    ]) {
        tokens.insert("intent_gap");
    }
    if contains_any(&["authorization", "権限", "授权"]) {
        tokens.insert("authorization_scope");
    }
    if contains_any(&["finalization", "finalization", "收尾", "終結"]) {
        tokens.insert("finalization");
    }
    if contains_any(&["Human next step", "人的下一步", "人間の次のアクション"]) {
        tokens.insert("next_action");
    }
    if language == "ja" && text.contains("人間の判断状態:") {
        tokens.insert("localized_human_decision_label");
    }
    tokens
}

fn registry_semantic_tokens(check_id: &str) -> Vec<String> {
    let matrix: serde_json::Value = serde_json::from_str(include_str!(
        "../../../docs/reference/collaboration-scenario-matrix.json"
    ))
    .expect("scenario matrix JSON");
    matrix["executableCheckRegistry"][check_id]["expectedSemantics"]["semanticTokens"]
        .as_array()
        .unwrap_or_else(|| panic!("{check_id} semanticTokens missing"))
        .iter()
        .map(|token| token.as_str().expect("semantic token string").to_owned())
        .collect()
}

/// Automated check for collaboration-language semantic invariant 5 (see
/// docs/reference/collaboration-language-contract.md and
/// docs/reference/collaboration-invariant-coverage.md): the same fact must
/// not contradict itself across the CLI and MCP entry points.
///
/// Unlike `crates/cockpit-mcp/tests/rpc.rs`'s `*_with_cli_parity` tests,
/// which call the MCP handler as an in-process library function and compare
/// it against another in-process rendering call, this test spawns the real
/// `ai-cockpit` CLI binary as a separate subprocess (a genuine second entry
/// point, not just a second function call) and compares its live stdout
/// against a live MCP handler call against the identical repository
/// fixture.
#[test]
fn cli_subprocess_and_mcp_handler_agree_on_the_same_outcome() {
    let binary = env!("CARGO_BIN_EXE_ai-cockpit");
    let id = "WI-CLI-MCP-PARITY";
    let repo = repository();

    run_json(binary, repo.path(), &["attach"]);
    run_json(
        binary,
        repo.path(),
        &[
            "start",
            "--id",
            id,
            "--intent",
            "prove CLI and MCP agree on the same Outcome for the same repository and Work Item",
            "--goal",
            "a cross-entry-point regression is caught automatically instead of relying on manual review",
            "--scope",
            "README.md",
            "--authority",
            "authorized",
            "--acceptance",
            "A1: CLI stdout outcome and MCP structuredContent.outcome are identical",
            "--required-evidence",
            "verification",
        ],
    );
    common::plan(binary, repo.path(), id);
    run_json(
        binary,
        repo.path(),
        &[
            "preflight",
            "--contract",
            &format!(".ai/work-items/active/{id}.contract.json"),
        ],
    );
    run_json(binary, repo.path(), &["checkpoint", "--id", id]);
    run_json(
        binary,
        repo.path(),
        &["verify", "--work-item", id, "--command", "true"],
    );

    // Entry point 1: the real CLI binary, as a separate subprocess.
    let cli_output = Command::new(binary)
        .args(["work-item", "outcome", "--repo"])
        .arg(repo.path())
        .args(["--id", id, "--json"])
        .output()
        .expect("cli outcome");
    assert!(
        cli_output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&cli_output.stderr)
    );
    let cli_outcome: serde_json::Value =
        serde_json::from_slice(&cli_output.stdout).expect("cli outcome JSON");

    // Entry point 2: the production MCP handler, called in-process against
    // the identical repository with a RuntimeContext bound to the exact
    // same binary under test.
    let runtime_context = current_runtime_context(binary);
    let response = cockpit_mcp::handle_request_for_repo(
        &serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "tools/call",
            "params": {"name": "work_item_outcome", "arguments": {"workItemId": id}}
        }),
        repo.path(),
        &runtime_context,
    );
    assert_eq!(
        response["result"]["isError"], false,
        "MCP call failed: response={response}"
    );
    let mcp_outcome = &response["result"]["structuredContent"]["outcome"];

    assert_eq!(
        &cli_outcome, mcp_outcome,
        "CLI stdout outcome and MCP structuredContent.outcome diverged for the same repository and Work Item"
    );

    // The comparison above must not be vacuously true because both sides
    // failed to bind to the Work Item at all.
    assert_eq!(cli_outcome["workItemId"], id);
    assert_eq!(mcp_outcome["workItemId"], id);
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
        "machine Outcome must carry the same structured reason projection used by the human handoff: {cli_outcome}"
    );
    assert_eq!(
        cli_outcome["governanceReasons"],
        mcp_outcome["governanceReasons"]
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

    // Human-language parity is a separate assertion from machine JSON
    // equality. It checks the actual handoff text for stable facts and the
    // non-authorization consequence, while allowing translated labels.
    for (language, required_fragments) in [
        (
            "en",
            vec![
                "does not mean verification evidence is invalid",
                "intent-alignment evidence is insufficient",
            ],
        ),
        ("zh", vec!["不是验证证据无效", "意图对齐证据不足"]),
        (
            "ja",
            vec![
                "検証 evidence が無効という意味ではありません",
                "intent alignment evidence が不足しています",
            ],
        ),
    ] {
        let cli_handoff = human_cli_output(binary, repo.path(), id, language);
        let mcp_response = cockpit_mcp::handle_request_for_repo(
            &serde_json::json!({
                "jsonrpc": "2.0",
                "id": 2,
                "method": "tools/call",
                "params": {"name": "work_item_outcome", "arguments": {"workItemId": id, "language": language}}
            }),
            repo.path(),
            &runtime_context,
        );
        assert_eq!(mcp_response["result"]["isError"], false);
        let mcp_handoff = mcp_response["result"]["structuredContent"]["humanHandoff"]
            .as_str()
            .expect("MCP human handoff");
        assert_eq!(
            human_semantic_lines(&cli_handoff),
            human_semantic_lines(mcp_handoff)
        );
        assert_eq!(
            human_semantic_tokens(&cli_handoff, language),
            human_semantic_tokens(mcp_handoff, language),
            "language={language}: CLI and MCP human handoffs must preserve the same semantic token set"
        );
        let actual_tokens = human_semantic_tokens(&cli_handoff, language);
        for expected in registry_semantic_tokens("human_report_entrypoint_parity") {
            assert!(
                actual_tokens.contains(expected.as_str()),
                "language={language}: human handoff missed registry semantic token {expected}: {cli_handoff}"
            );
        }
        assert!(
            cli_handoff.contains("Outcome:"),
            "{language}: {cli_handoff}"
        );
        assert!(
            cli_handoff.contains("Human next step")
                || cli_handoff.contains("人的下一步")
                || cli_handoff.contains("人間の次のアクション"),
            "{language}: {cli_handoff}"
        );
        for fragment in required_fragments {
            assert!(
                cli_handoff.contains(fragment) && mcp_handoff.contains(fragment),
                "language={language}, fragment={fragment:?}, cli={cli_handoff}, mcp={mcp_handoff}"
            );
        }
    }
}

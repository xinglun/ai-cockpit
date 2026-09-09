use sha2::{Digest as ShaDigest, Sha256};
use std::{fs, path::Path, process::Command};

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
            "CLI stdout outcome and MCP structuredContent.outcome are identical",
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
}

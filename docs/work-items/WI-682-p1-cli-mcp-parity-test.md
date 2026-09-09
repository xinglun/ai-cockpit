---
author: AI Cockpit maintainers
title: "WI-682 — P1 CLI/MCP Outcome parity test"
description: "Closes the invariant-5 gap named by WI-681: a new integration test spawning the real CLI binary and calling the production MCP handler against the same repository, asserting their Outcome representations are identical."
audience: [maintainer, reviewer, adopter]
workItemId: WI-682-p1-cli-mcp-parity-test
status: implemented
authority: authorized
lastVerifiedBy: WI-682-p1-cli-mcp-parity-test
terminalArchive: .ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json
terminalVerification: .ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json
terminalFinalization: .ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json
terminalDecision: .ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json
---

[简体中文](WI-682-p1-cli-mcp-parity-test.zh-CN.md) · [日本語](WI-682-p1-cli-mcp-parity-test.ja.md)

# WI-682 — P1 CLI/MCP Outcome parity test

## Intent

Close the invariant-5 gap named in WI-681's coverage mapping
(`docs/reference/collaboration-invariant-coverage.md`, not yet on the
default branch as of this Work Item; see PR pending): "the same fact must
not contradict itself across CLI, MCP, a summary, and a full report" had no
true cross-process automated check. This Work Item adds exactly one new
integration test combining two already-proven existing patterns (subprocess
spawning from `crates/cockpit-cli/tests/outcome_handoff.rs`; in-process MCP
handler calls from `crates/cockpit-mcp/tests/rpc.rs`) rather than
introducing new test infrastructure, per explicit repository-owner
delegation to continue the AI Cockpit collaboration-language initiative.

## Boundary

This is a test-only Work Item. It adds exactly one new file,
`crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`. It does not modify any
production source file and does not modify any existing test file.

## Acceptance and lifecycle

- The new test spawns the real `ai-cockpit` binary as a subprocess for the
  CLI side and calls `cockpit_mcp::handle_request_for_repo()` in-process for
  the MCP side, against the identical repository fixture and Work Item, and
  asserts the two Outcome representations are exactly equal.
- `cargo test --locked -p cockpit-cli --test cli_mcp_outcome_parity`,
  `cargo fmt --check`, and `cargo clippy --tests -- -D warnings` all pass.
- `start → preflight → checkpoint → verify → finish → archive → close` is the
  governed route; `user_visible_benefit_not_declared` remains explicit.

## Evidence

- archive: `.ai/work-items/archive/WI-682-p1-cli-mcp-parity-test.contract.json`
- verification: `.ai/evidence/WI-682-p1-cli-mcp-parity-test.verification.json`
- finalization: `.ai/decisions/WI-682-p1-cli-mcp-parity-test.finalize.json` (pending merge)
- close: `.ai/decisions/WI-682-p1-cli-mcp-parity-test.close.json` (pending merge)

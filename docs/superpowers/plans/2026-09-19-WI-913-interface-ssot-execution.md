# WI-913 Interface SSOT Execution Plan

> **For agentic workers:** Use the red-green verification sequence below for each change. This plan is intentionally limited to the `work-item outcome` discovery surface.

**Goal:** Make the protocol-owned Outcome interface facts the only maintained source for the CLI help, MCP schema, and deterministic discovery projection.

**Architecture:** `cockpit-protocol` owns shared parameter descriptions and reusable parameter specs. The CLI derives its Clap help text from those constants, while MCP and the generated reference projection continue to consume the same protocol tables. Agent adapters pass the active conversation language explicitly to the human Outcome projection; Contract bytes and machine fields remain language-neutral. No lifecycle, authorization, host delivery, or release behavior changes.

**Tech Stack:** Rust workspace, Clap derive, serde/JSON, existing MCP schema projection, and the checked-in tri-language generated-region tests.

**Spec:** `docs/superpowers/specs/2026-09-17-interface-discovery-design.md`

## Global Constraints

- Do not modify Outcome delivery, lifecycle, authorization, verification, receipt, object repositories, or release publication.
- Keep discovery deterministic and read-only; it must not inspect Work Item history or start verification subprocesses.
- Use `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR="$HOME/.cache/ai-cockpit-verify-target"` for Rust verification.
- Preserve human-written text outside the generated interface-facts markers.
- The active conversation language is selected by the adapter (`--language` or MCP `language`); locale fallback is only for direct calls that cannot observe the conversation.

### Task 1: Lock the regression

**Files:** `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`

- Add a test that runs the production `work-item outcome --help` command and checks the delivery, JSON, view, and language descriptions against `work_item_outcome_interface_specs("cli")`.
- Run the exact test before the implementation change; it must fail because the existing Clap help strings are duplicated and differ from protocol descriptions.

### Task 2: Remove duplicated protocol facts

**Files:** `crates/cockpit-protocol/src/interface_description.rs`, `crates/cockpit-protocol/src/lib.rs`, `crates/cockpit-cli/src/main.rs`

- Define exported protocol-owned description constants and reusable `InterfaceParameterSpec` values for shared delivery, view, and language parameters.
- Use those constants in the CLI `Outcome` Clap attributes and in both CLI/MCP interface tables.
- Keep transport-specific identity/alias facts separate where their wire names differ.

### Task 3: Prove both projections

**Files:** `crates/cockpit-protocol/tests/interface_description.rs`, `crates/cockpit-mcp/tests/rpc.rs`

- Assert common CLI/MCP parameter facts are equal for delivery, view, and language.
- Retain the existing MCP schema projection test, which compares every exposed property with the protocol table.
- Run protocol, CLI, and MCP focused tests, then formatting and documentation parity checks.

### Task 4: Preserve conversation-language delivery

**Files:** `crates/cockpit-agent/src/lib.rs`, `crates/cockpit-agent/tests/install.rs`, `crates/cockpit-mcp/src/lib.rs`, `docs/reference/outcome-report.md`, `docs/reference/outcome-report.zh-CN.md`, `docs/reference/outcome-report.ja.md`

- Require managed Agent guidance to pass the active dialogue language explicitly as CLI `--language <en|zh|ja>` or MCP `language`.
- Keep direct CLI/MCP locale fallback as a documented fallback only; never infer a human Outcome language from the Contract source language.
- Cover the guidance in the adapter installation regression and retain identical facts across the three reference pages.

### Task 5: Governed delivery

- Run Contract preflight/checkpoint/verification with the shared target directory.
- Record the Summary and human Outcome through Runtime commands; never hand-edit generated lifecycle records.
- Open one reviewed PR through the `xinglun` GitHub account, wait for required checks, merge, synchronize main, and remove only this Work Item's branch/worktree.
- Do not publish a release from this Work Item.

# WI-881 Interface Discovery Implementation Plan
> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox syntax and include verification after each change.

**Goal:** Add a deterministic, shared interface description for the `work-item outcome` command family through the existing capability discovery entry, keep CLI/MCP/Agent guidance facts aligned, and prove reduced duplicate maintenance without changing governance behavior.

**Architecture:** `cockpit-protocol` owns a pure versioned `InterfaceDescription` and JSON/Markdown renderers. CLI `capability show --surface work-item-outcome` and MCP `capability_show` delegate to it. Tests compare the projection with actual Clap and MCP schemas. Three reference pages contain only marked generated facts; human explanation remains outside the markers. Agent guidance points to the read-only discovery path and existing ownership tests prove no silent overwrite.

**Tech Stack:** Rust workspace, Clap, serde/serde_json, MCP stdio adapter, Markdown reference pages, `CARGO_INCREMENTAL=0` with shared verification target.

## Global Constraints

- Work only in the dedicated `codex/wi-881-interface-discovery` worktree.
- Do not modify WI-880, Sentinel, any object repository, global Agent/MCP configuration, or generated Runtime status/archive evidence by hand.
- Do not change Outcome lifecycle, authorization, verification, receipt, or host-delivery semantics.
- Do not add a new governance layer or a broad documentation generator.
- Use `apply_patch` for source and documentation edits.

## Task 1: Lock the trial contract and the failing checks

- [x] Confirm latest `origin/main`, Runtime `inspect/status/doctor`, BAML reference commit, and the selected existing discovery entry.
- [x] Record the WI Contract, scenario coverage, checkpoint, and design spec.
- [ ] Add failing protocol/CLI/MCP tests for the required description fields and the `capability show --surface work-item-outcome` route.
- [ ] Run only the new focused tests and preserve the red output as the TDD starting point.

## Task 2: Implement the shared protocol description

- [ ] Add versioned serializable description structs and the pure `work_item_outcome_interface_description()` projection in `cockpit-protocol`.
- [ ] Add deterministic JSON and localized Markdown rendering with no filesystem, Git, clock, network, or subprocess access.
- [ ] Add protocol unit tests for requiredness, defaults, enums, aliases, schema/runtime versions, and repeatable output.
- [ ] Run `cargo test --locked -p cockpit-protocol --lib` with the shared target.

## Task 3: Expose the projection through existing CLI/MCP discovery

- [ ] Extend `capability show` with optional `surface`, `format`, and language selection while preserving the no-argument registry output.
- [ ] Extend MCP `capability_show` schema/validation/handler with the same optional discovery request and delegate to the protocol projection.
- [ ] Add parser/schema parity tests that fail if the declared `view`, `delivery`, identity aliases, defaults, or enum values drift from actual behavior.
- [ ] Assert the discovery branch performs no Work Item-history read and starts zero verification subprocesses.
- [ ] Run focused `cockpit-cli` and `cockpit-mcp` tests.

## Task 4: Generate and protect the reference facts

- [ ] Insert marked generated interface-fact regions in English, Simplified Chinese, and Japanese command references, retaining human-written explanations around them.
- [ ] Add deterministic parity checks that render twice, compare digests, and ensure only the marked regions match the shared renderer.
- [ ] Update first-start guidance to point Agents to the read-only discovery route and distinguish description from authority.
- [ ] Run the documentation acceptance/parity checks that are in scope; do not run an unrelated full release gate.

## Task 5: Agent ownership evidence and handoff facts

- [ ] Re-run existing install/doctor tests for current, owned-old, modified, unknown-ownership, and repeated-install states.
- [ ] Add only the missing first-start discovery assertion; if implementation already satisfies the ownership contract, record the no-change evidence in the Summary.
- [ ] Record before/after manual edit locations, operation count, and measured focused-test durations. Keep benefit unknown unless measured.

## Task 6: Verification and governed delivery

- [ ] Run `cargo fmt --all -- --check` and all cheap static/scope checks first.
- [ ] Run focused protocol, CLI, MCP, Agent, and documentation tests with `CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=$HOME/.cache/ai-cockpit-verify-target`.
- [ ] Run the Contract-required `cargo test --locked --workspace` only after focused gates pass; capture exit code, logs, and timing.
- [ ] Refresh Runtime evidence, update the Summary/Outcome through Runtime commands, run `finish → archive → close`, and perform exact branch/worktree cleanup.
- [ ] Open one PR from the dedicated branch with `gh` under the active `xinglun` account, wait for required hosted checks, merge only the reviewed PR, synchronize main, and verify no provider auto-deletion bypassed finalization.
- [ ] Do not publish a release from this WI. Release remains the final step after WI-880, the four-direction convergence audit, Rust/toolchain/dependency work, Issue #851, and all required issue/WI closure evidence.

## Review checklist

- [ ] One shared interface fact source; no parallel hand-maintained registry.
- [ ] CLI, MCP, JSON, Markdown, and three-language facts agree.
- [ ] Human prose is not overwritten.
- [ ] Description path is read-only and does not infer permissions or lifecycle state.
- [ ] Agent ownership safety is preserved.
- [ ] Outcome behavior and delivery semantics are unchanged.
- [ ] Outcome states measured maintenance cost and explicit unknowns.

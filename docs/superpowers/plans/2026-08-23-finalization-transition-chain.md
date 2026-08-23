# Finalization Transition Chain Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add immutable, typed post-merge finalization transitions and make verification/close consume the unique latest head.

**Architecture:** Preserve the legacy canonical receipt as the root, append digest-addressed typed envelopes, resolve a strict linear chain, and bind close to its verified head. Existing canonical-only repositories require no migration.

**Tech Stack:** Rust, serde, cap-std filesystem validation, Git-backed repository tests, clap CLI integration tests.

**Spec:** `docs/superpowers/specs/2026-08-23-finalization-transition-chain-design.md`

## Global Constraints

- Never overwrite WI-190 or any canonical finalization receipt.
- Reject ambiguous, foreign, stale, malformed, or symlinked chains.
- Keep legacy deleted/retained canonical receipts compatible.
- Run all repository-bound Runtime commands with explicit `--repo`.

---

### Task 1: Typed transition protocol

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Test: `crates/cockpit-protocol/tests/resource_finalization.rs`

- [ ] Add failing tests for a typed successor, stale predecessor, identity drift, merge regression, and deleted terminal state.
- [ ] Run the protocol test and confirm failure because transition types/validation are absent.
- [ ] Implement `ResourceFinalizationTransitionReceipt` and transition validation.
- [ ] Run the protocol test to green.

### Task 2: Append-only repository resolver

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Create: `crates/cockpit-repository/tests/resource_finalization_transition.rs`

- [ ] Add failing real-repository tests for append/head resolution, WI-190 topology, replay, forks, stale sequence, symlink/malformed input, local postconditions, and legacy compatibility.
- [ ] Run the repository test and confirm canonical-only behavior fails it.
- [ ] Implement digest-suffixed atomic append and strict unique-head resolution.
- [ ] Make `finalize-verify` and close consume and bind the resolved head.
- [ ] Run repository tests to green.

### Task 3: CLI and documentation parity

**Files:**
- Modify: `crates/cockpit-cli/tests/lifecycle.rs`
- Modify: `docs/reference/agent-workflow*.md`, `docs/reference/commands*.md`, `docs/reference/reference-parity*.md`
- Create: `docs/work-items/WI-191-finalization-transition-chain*.md`

- [ ] Add a failing CLI lifecycle assertion for `appended` transition output and latest-head verification.
- [ ] Implement any required CLI projection changes and run the lifecycle test.
- [ ] Document the transition chain and WI-190 recovery boundary in English, Chinese, and Japanese.
- [ ] Run documentation/static gates.

### Task 4: Governed delivery

- [ ] Run focused tests and `cargo test --locked --workspace`.
- [ ] Bind the PR resource context with `finalize-plan`, rerun verification, finish, and archive.
- [ ] Record a pre-merge blocked finalization receipt without rewriting it.
- [ ] Commit, push, open the PR, and report hosted CI without merging.

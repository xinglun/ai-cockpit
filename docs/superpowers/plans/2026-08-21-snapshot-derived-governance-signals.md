# Snapshot-Derived Governance Signals Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Derive untrusted-material, test-weakening, and coverage-weakening governance inputs from one immutable repository snapshot.

**Architecture:** `cockpit-git` captures process-local bounded change facts during its existing four-call snapshot. `cockpit-repository` analyzes those facts without IO and the central decision adapter feeds Core for CLI/MCP and terminal lifecycle operations.

**Tech Stack:** Rust, Git CLI fixed argv, Serde, existing CLI/MCP integration tests.

**Spec:** `docs/superpowers/specs/2026-08-21-snapshot-derived-governance-signals-design.md`

## Global Constraints

- Keep `RepositorySnapshot.git_calls == 4`.
- Do not serialize raw changed text through CLI, MCP, evidence, or cache files.
- Inspect at most 262,144 bytes per changed file and fail closed for relevant uninspectable content.
- Do not add a second Git scan or checker subprocess.
- Do not commit, push, tag, publish, or install `.ai` without explicit authorization.

---

### Task 1: Immutable change evidence

**Files:**
- Modify: `crates/cockpit-git/src/lib.rs`
- Modify: `crates/cockpit-git/tests/repository.rs`
- Modify: `crates/cockpit-repository/tests/evolution.rs`

**Interfaces:**
- Produces: `ChangeEvidence`, `ChangeKind`, `ChangeContentState`, and `RepositorySnapshot.change_evidence`.

- [ ] Write failing tests for tracked line facts, untracked text, size state, four Git calls, and serialization redaction.
- [ ] Run the focused Git tests and confirm missing change evidence is the RED cause.
- [ ] Parse zero-context diff facts and reuse already-read changed bytes.
- [ ] Run focused tests and retain GREEN evidence.

### Task 2: Deterministic governance analysis

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Create: `crates/cockpit-repository/tests/governance_signals.rs`
- Modify: `crates/cockpit-core/src/lib.rs`

**Interfaces:**
- Produces: `GovernanceSignalAssessment` and `derive_governance_signals(&RepositorySnapshot)`.
- Consumes: process-local change evidence only; performs no IO.

- [ ] Write failing tests for prompt injection, security-test deletion, coverage lowering, safe test additions, and uninspectable relevant material.
- [ ] Implement minimal deterministic detectors and explicit unknown mappings.
- [ ] Replace the three hard-coded booleans in `governance_decision_for_contract`.
- [ ] Run focused repository and Core tests.

### Task 3: Adapter and disclosure regression

**Files:**
- Modify: `crates/cockpit-cli/tests/lifecycle.rs`
- Modify: `crates/cockpit-mcp/tests/rpc.rs`

**Interfaces:**
- Consumes: the central repository decision adapter.
- Produces: end-to-end proof that CLI/MCP agree and observation JSON does not leak raw text.

- [ ] Add failing CLI and MCP tests for derived signals and serialized redaction.
- [ ] Make only central wiring changes required for both adapters to pass.
- [ ] Run focused CLI/MCP lifecycle and RPC tests.

### Task 4: Documentation and full verification

**Files:**
- Create: `docs/work-items/WI-28.md`
- Create: `docs/work-items/WI-28.zh-CN.md`
- Create: `docs/work-items/WI-28.ja.md`

**Interfaces:**
- Produces: traceable Bootstrap WI evidence and final local acceptance state.

- [ ] Record RED/GREEN evidence and changed files in all three Work Item records.
- [ ] Run `cargo test --workspace` twice.
- [ ] Run Clippy, fmt, diff, workflow YAML, and multilingual checks.
- [ ] Re-run executable V1 Oracle and report hosted/integration/release limitations honestly.

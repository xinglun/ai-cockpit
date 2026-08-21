# Fail-Closed Evidence Reuse Planner Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:executing-plans and superpowers:test-driven-development. Execute
> each task with RED evidence before production changes.

**Goal:** Replace caller-declared verification skips with receipt-validated,
dependency-aware, fail-closed planning and observable bounded execution.

**Architecture:** `cockpit-evidence` classifies a complete content/diff/
environment identity as Fresh, Stale, or Unknown. `cockpit-verification`
requires an explicit Protected/Reusable/NeverReuse policy, derives the actual
command identity, plans dependency-aware actions, and spawns only commands
whose action is Execute after their executed dependencies complete.

**Tech Stack:** Rust 1.94, serde/JSON, `cockpit-core::Digest`, Cargo tests.

**Spec:**
`docs/superpowers/specs/2026-08-21-fail-closed-evidence-reuse-planner-design.md`

## Global Constraints

- Reuse only a passed, exact, unexpired, internally valid composite receipt.
- Missing or invalid evidence executes; protected nodes always execute.
- Derive command identity from the command actually passed to the executor.
- A dependency rerun forces every dependent candidate to execute.
- Do not create `.ai/`, attach, commit, push, tag, publish, or release.
- Preserve all unrelated dirty-worktree changes.

---

### Task 1: Composite Receipt Identity and Pure Decision

**Files:**
- Modify: `crates/cockpit-evidence/Cargo.toml`
- Modify: `crates/cockpit-evidence/src/lib.rs`
- Test: `crates/cockpit-evidence/tests/reuse.rs`

**Interfaces:**
- Produces: `DiffIdentity`, `EvidenceContext`, `ReusableReceipt`,
  `ReuseState`, `ReuseAction`, `ReuseReason`, and `decide_reuse`.
- Consumers: Task 2 typed reuse candidate and planner.

- [ ] Add tests for one exact Fresh/Reuse composite receipt.
- [ ] Add a table test mutating every identity dimension to Stale/Execute.
- [ ] Add missing, failed, future, malformed, tampered, expired, and protected
      fail-closed tests.
- [ ] Run the focused test and capture compile/behavior RED.
- [ ] Implement deterministic receipt ids, validation, and the pure decision.
- [ ] Run the focused suite to GREEN.

### Task 2: Typed Command Candidate and Dependency Planner

**Files:**
- Modify: `crates/cockpit-verification/Cargo.toml`
- Modify: `crates/cockpit-verification/src/lib.rs`
- Test: `crates/cockpit-verification/tests/execution.rs`

**Interfaces:**
- Consumes: Task 1 receipt, context, and decision types.
- Produces: mandatory typed reuse policy/candidate, derived command digest,
  planned commands, dependency-ready bounded scheduling, per-node results, and
  deterministic time-aware execution entry point.

- [ ] Add a test where a Fresh receipt skips an actually failing command and
      prove no process was spawned.
- [ ] Add stale, missing, and protected execution tests with metrics.
- [ ] Add an upstream-rerun/downstream-fresh dependency test that must execute
      both commands.
- [ ] Run the focused test and capture the missing typed API RED.
- [ ] Remove `reuse: bool` and `with_reuse(bool)` from production code.
- [ ] Require explicit Protected/Reusable/NeverReuse policy at construction;
      no default protected bool may authorize or omit a skip boundary.
- [ ] Implement command digest derivation and dependency-aware planning.
- [ ] Schedule only ready Execute commands; dependent nodes wait while
      independent nodes remain bounded-parallel.
- [ ] Preserve per-node action/state/reason/receipt/satisfaction results and
      distinguish successful spawns from spawn failures.
- [ ] Run graph and execution suites to GREEN.

### Task 3: Consumer Compatibility and Mutation Evidence

**Files:**
- Modify as required: CLI/MCP verification output tests.
- Modify: `docs/work-items/WI-32.md`
- Modify: `docs/work-items/WI-32.zh-CN.md`
- Modify: `docs/work-items/WI-32.ja.md`

**Interfaces:**
- Consumes: the new execution receipt metrics.
- Produces: unchanged protected CLI/MCP execution plus truthful local evidence.

- [ ] Run affected CLI/MCP focused tests and preserve protected behavior.
- [ ] Require zero production occurrences of the old naked reuse flag/API.
- [ ] Temporarily weaken one binding mismatch and prove a focused test fails;
      restore the correct implementation and rerun GREEN.
- [ ] Run two workspace test passes, warnings-denied Clippy, rustfmt, diff,
      workflow YAML, multilingual checks, and the locked V1 Oracle.
- [ ] Update all three WI outcomes with exact evidence and leave `.ai/` absent.
- [ ] Open the next Work Item for cross-process receipt/profile integration;
      do not implement it in WI-32.

# Cross-Process Evidence Reuse Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use
> superpowers:executing-plans and superpowers:test-driven-development. Preserve
> WI-32 fail-closed semantics and capture RED before production changes.

**Goal:** Make confirmed reusable verification evidence survive process exit
and drive the same safe CLI/MCP execution service.

**Architecture:** The repository layer derives one immutable profile-bound
verification context and owns an atomic content-addressed receipt store. A
shared service loads candidates, invokes the WI-32 planner/executor, and writes
new passed receipts. CLI/MCP remain thin adapters.

**Spec:**
`docs/superpowers/specs/2026-08-21-cross-process-evidence-reuse-integration-design.md`

## Global Constraints

- Reuse only exact commands explicitly verified by a calibrated profile.
- Load repository/snapshot/profile once and share immutable facts.
- Missing or untrusted storage executes; never auto-upgrade evidence to Fresh.
- Protected nodes execute on every process invocation.
- No runtime source/scripts/schemas are copied into attached repositories.
- Do not create this repository's `.ai/`, commit, push, tag, publish, or release.

---

### Task 1: Schema Evolution and Context Derivation

**Files:**
- Modify: `crates/cockpit-evidence/src/lib.rs` and tests.
- Modify: `crates/cockpit-repository/src/lib.rs` and focused tests.

- [x] Add profile digest to the receipt context and fail closed on old/unknown
      schema candidates.
- [x] Add RED tests for deterministic content/diff/environment/scope/
      governance/toolchain/policy/profile/stage/runner derivation.
- [x] Derive immutable authorization facts once, then revalidate raw identity
      and repository snapshots without authorizing post-run content.
- [x] Reject missing HEAD, malformed profile, unverified commands, and changing
      input facts as Unknown/Execute.

### Task 2: Atomic Content-Addressed Receipt Store

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`.
- Test: new receipt-store integration suite.

- [x] Add RED tests for empty, valid, malformed, tampered, missing, symlinked,
      wrong-repository, and wrong-profile stores.
- [x] Write immutable receipt files before atomically replacing the validated
      index.
- [x] Read only the index and referenced receipts; do not scan receipt history.
- [x] Prove failed writes never authorize reuse and never corrupt the previous
      valid index.

### Task 3: Output-Bound Execution Receipts

**Files:**
- Modify: `crates/cockpit-verification/src/lib.rs` and tests.

- [x] Add bounded stdout/stderr/status capture and deterministic output digest.
- [x] Add RED tests that failed spawn, non-zero exit, malformed/truncated
      output state, or post-command binding drift creates no passed receipt.
- [x] Return passed receipt candidates only for successful real executions.
- [x] Preserve bounded parallelism, dependency completion ordering, and exact
      per-node audit results.

### Task 4: Shared Verification Service and Adapters

**Files:**
- Create or modify: shared repository verification service module.
- Modify: `crates/cockpit-cli/src/main.rs` and tests.
- Modify: `crates/cockpit-mcp/src/lib.rs` and tests.

- [x] Add a two-process CLI RED test: first run executes, unchanged calibrated
      second run reuses with zero process, and protected nodes still execute.
- [x] Add per-binding process-to-process invalidation tests.
- [x] Route CLI and MCP through the same service; require parity of decisions,
      receipt ids, per-node results, and metrics.
- [x] Keep explicit/unconfirmed commands NeverReuse.

### Task 5: Adversarial, Cost, and Complete Verification

**Files:**
- Modify: `docs/work-items/WI-33.md`, `.zh-CN.md`, `.ja.md`.

- [x] Mutation-test store identity, profile authorization, and one binding.
- [x] Prove actual call-count reduction and report git/files/hash/process costs.
- [x] Run focused suites, two workspace runs, warnings-denied Clippy, rustfmt,
      diff, workflow YAML, multilingual checks, and locked V1 Oracle.
- [x] Confirm this development repository still has no `.ai/` and report only
      local implementation truth.

Final independent review: **Ready to merge**, with zero Critical and zero
Important findings. The dedicated Windows runtime CI job remains a required
pre-integration gate because it cannot run on the local macOS host.

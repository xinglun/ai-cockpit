# Executable V1 Behavioral Oracle Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the self-declared V1 parity claim with an executable, commit-locked external V1 semantic Oracle for all fourteen conformance cases.

**Architecture:** A test-only Python adapter calls governance functions from an external V1 checkout and emits the seven canonical V2 comparison fields. A Rust integration test validates reference identity, executes the adapter without a shell, and compares its output to the corpus; CI supplies the exact reference checkout.

**Tech Stack:** Rust integration tests, Python 3 external V1 runtime, JSON, GitHub Actions.

**Spec:** `docs/superpowers/specs/2026-08-21-executable-v1-behavioral-oracle-design.md`

## Global Constraints

- Do not copy V1 runtime files, schemas, installer code, or Make implementation.
- Do not introduce Python or network requirements into the Rust binary or adopter repositories.
- Require V1 Git HEAD to equal `tests/conformance/v1-reference.lock`.
- Compare all seven canonical semantic fields for all fourteen cases.
- Do not read `expected.json` from the V1 adapter.
- Do not commit, push, tag, or publish without explicit user authorization.

---

### Task 1: Commit-locked Rust Oracle boundary

**Files:**
- Create: `crates/cockpit-core/tests/v1_oracle.rs`
- Create: `tests/conformance/v1_oracle.py`

**Interfaces:**
- Consumes: `AI_COCKPIT_V1_ROOT`, `v1-reference.lock`, fixture directory.
- Produces: one JSON array containing a seven-field semantic result per case.

- [ ] Write a Rust integration test that fails because the adapter does not yet exist.
- [ ] Run `AI_COCKPIT_V1_ROOT=<locked-checkout> cargo test -p cockpit-core --test v1_oracle -- --ignored --nocapture` and confirm the missing-adapter failure.
- [ ] Implement reference-root canonicalization, Git HEAD comparison, fixed-argv Python execution, JSON decoding, exact case-set validation, and seven-field comparison.
- [ ] Add a wrong-commit test using a temporary Git repository and confirm it fails before Python execution.
- [ ] Run the focused Rust test and retain its RED/GREEN evidence.

### Task 2: Fourteen real V1 probes

**Files:**
- Modify: `tests/conformance/v1_oracle.py`
- Modify: conformance fixture expectations only when executable V1 evidence proves the previous expectation wrong.

**Interfaces:**
- Consumes: external V1 `scripts/` modules and V2 fixture semantic facts.
- Produces: canonical `decisionState`, `blockers`, `unknowns`, `safeActions`, `requiredChecks`, `authority`, and `outcomeState`.

- [ ] Implement scope, evidence-state, authority, input-trust, status, unsupported-claim, archive, provider, test-weakening, and coverage probes.
- [ ] Ensure the adapter rejects an unknown case and never opens `expected.json`.
- [ ] Run the Oracle test against reference commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`; treat semantic mismatches as RED.
- [ ] Reconcile V2 expected semantics only from captured V1 outputs and rerun until all fourteen comparisons pass.
- [ ] Mutation-check one expected field and confirm the Oracle test fails, then restore it and confirm GREEN.

### Task 3: Honest manifest, CI, and documentation

**Files:**
- Modify: `tests/conformance/manifest.json`
- Modify: `tests/conformance/README.md`
- Modify: `tests/conformance/README.zh-CN.md`
- Modify: `tests/conformance/README.ja.md`
- Modify: `.github/workflows/ci.yml`
- Create: `docs/work-items/WI-27.md`
- Create: `docs/work-items/WI-27.zh-CN.md`
- Create: `docs/work-items/WI-27.ja.md`

**Interfaces:**
- Consumes: the locked commit and focused Oracle test.
- Produces: distinct offline/executable modes and a mandatory CI Oracle job.

- [ ] Replace the false global `runtimeInvoked` claim with explicit offline and executable modes.
- [ ] Add a CI job that reads the lock, fetches that exact commit, and runs only the executable Oracle boundary.
- [ ] Document that normal builds remain offline and that Gate B proof comes from the dedicated job.
- [ ] Record RED/GREEN evidence and honest residual limitations in WI-27 across all three languages.
- [ ] Run workflow syntax parsing and the three-language counterpart check.

### Task 4: Full verification

**Files:**
- Modify: `docs/work-items/WI-27*.md` with final evidence.

**Interfaces:**
- Consumes: all WI-27 implementation artifacts.
- Produces: final local acceptance evidence without integration or release claims.

- [ ] Run the executable V1 Oracle against the locked checkout.
- [ ] Run `cargo test --workspace` twice.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo fmt --all --check` and `git diff --check`.
- [ ] Parse every workflow YAML file and inspect `git status --short --branch`.
- [ ] Update WI-27 Outcome without claiming self-governance, hosted CI, merge, or release completion.

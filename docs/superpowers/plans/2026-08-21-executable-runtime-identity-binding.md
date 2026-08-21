# Executable Runtime Identity Binding Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Bind every runtime identity claim to the exact executing AI Cockpit binary.

**Architecture:** The CLI computes one streaming SHA-256 `RuntimeContext` from its current executable and injects that immutable context into CLI and MCP consumers. Repository evidence stores the injected identity; no adapter may reconstruct it from a version label.

**Tech Stack:** Rust 1.94, sha2, hex, Clap, serde/JSON, Cargo integration tests.

**Spec:** `docs/superpowers/specs/2026-08-21-executable-runtime-identity-binding-design.md`

## Global Constraints

- Keep Runtime Version separate from Repository Protocol Version.
- Hash the actual executing file once per CLI invocation.
- Fail closed on executable identity errors; never fall back to a label hash.
- Do not create `.ai/`, commit, push, tag, publish, or release.
- Preserve all unrelated dirty-worktree changes.

---

### Task 1: Exact CLI Runtime Context

**Files:**
- Create: `crates/cockpit-cli/src/runtime_identity.rs`
- Modify: `crates/cockpit-cli/Cargo.toml`
- Modify: `crates/cockpit-cli/src/main.rs`
- Test: `crates/cockpit-cli/tests/inspect.rs`

**Interfaces:**
- Produces: `runtime_identity::load_current() -> anyhow::Result<cockpit_protocol::RuntimeContext>` and `runtime_identity::load(&Path) -> anyhow::Result<RuntimeContext>`.
- Consumers: CLI dispatch and Task 2 MCP injection.

- [ ] Add an inspect integration assertion whose hand-computed binary SHA-256 must equal `runtimeDigest`.
- [ ] Run `cargo test -p cockpit-cli --test inspect` and record the expected mismatch against the version-string hash.
- [ ] Implement streaming executable hashing and construct `RuntimeContext` with package and protocol constants.
- [ ] Construct the context once in `run`; replace inspect and doctor literals with context fields.
- [ ] Add a unit test that a missing executable path returns an error.
- [ ] Run inspect and doctor focused tests to green.

### Task 2: Explicit MCP Runtime Context

**Files:**
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Modify: `crates/cockpit-mcp/tests/rpc.rs`
- Modify: `crates/cockpit-cli/src/main.rs`
- Test: `crates/cockpit-cli/tests/mcp.rs`

**Interfaces:**
- Consumes: `&cockpit_protocol::RuntimeContext` from Task 1.
- Produces: context-requiring MCP request and stdio serving functions.

- [ ] Add failing MCP tests asserting injected version/digest in initialize and verification evidence.
- [ ] Run MCP and CLI MCP focused tests and record placeholder/missing-context failures.
- [ ] Thread `&RuntimeContext` through bound/unbound request handlers and stdio servers.
- [ ] Pass the one CLI context into both MCP modes and verification evidence recording.
- [ ] Run MCP and CLI MCP focused tests to green.

### Task 3: CLI Verification Evidence Binding

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-cli/tests/verify.rs`
- Test: `crates/cockpit-cli/tests/lifecycle.rs`

**Interfaces:**
- Consumes: the invocation `RuntimeContext`.
- Produces: verification evidence with exact `runtimeVersion` and `runtimeDigest`.

- [ ] Add a failing integration assertion comparing stored CLI verification evidence with the binary's independently computed digest.
- [ ] Replace the remaining CLI placeholder identity with context fields.
- [ ] Run verification and lifecycle focused tests to green.
- [ ] Search production Rust sources and require zero version-string digest placeholders.

### Task 4: Documentation and Complete Verification

**Files:**
- Modify: `docs/work-items/WI-31.md`
- Modify: `docs/work-items/WI-31.zh-CN.md`
- Modify: `docs/work-items/WI-31.ja.md`

**Interfaces:**
- Consumes: focused RED/GREEN and full-gate evidence.
- Produces: truthful local outcome without integration or release claims.

- [ ] Perform mutation checks for inspect and verification identity propagation.
- [ ] Run two independent `cargo test --workspace --quiet` executions.
- [ ] Run Clippy with warnings denied, rustfmt, `git diff --check`, Python compilation, workflow YAML parsing, multilingual checks, and locked V1 Oracle.
- [ ] Measure warm status startup after executable hashing and record the observed median without weakening the existing gate.
- [ ] Update all three WI-31 outcomes and leave WI-30 readiness-only.

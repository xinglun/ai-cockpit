# Governance Scaffolding Without Governance Decisions Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add deterministic repository and Work Item scaffolding while preserving explicit repository binding and preventing invented governance decisions.

**Architecture:** The shared Rust Core remains request-scoped. Repository scaffolding and Work Item creation use one repository-owned API; the transitional `start` command delegates to that API. `profile propose` is a read-only candidate projection. Provider installation remains a later explicit adapter operation.

**Tech Stack:** Rust workspace, clap, serde/serde_json, existing Repository Snapshot and Contract validator, trilingual Markdown documentation.

**Spec:** `docs/superpowers/specs/2026-08-21-governance-scaffolding-without-decisions-design.md`

## Global Constraints

- Every CLI operation requires an explicit `--repo`; no ambient current-project state.
- Attach creates only repository-local protocol scaffold and discovery facts; never provider files, user-global MCP settings, or governance decisions.
- Auto-filled Work Item fields must be deterministically derived from the current snapshot/profile.
- Human-owned fields remain empty or `unknown`; skeletons are `not_ready`/`unknown`, never positive completion states.
- `profile propose` cannot write formal profile baseline bytes or digest.
- Keep `cockpit.toml` as TOML and keep the Rust Core external/shared.

---

### Task 1: Define strict discovery manifest and scaffold data contracts

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Modify: `crates/cockpit-protocol/tests/*.rs` (nearest protocol serialization test)
- Modify: `crates/cockpit-repository/src/lib.rs`

**Interfaces:**
- Add a strict `AgentInterfaceManifest` carrying schema/protocol version, stable `repositoryId`, manifest-parent binding, capability list, and an unconfigured adapter state.
- Add a repository helper that derives the semantic attached profile digest and snapshot-derived Work Item facts.
- Add one shared contract-scaffold function consumed by both `start` and `work-item new`.

- [x] Write failing serialization tests for manifest unknown fields and stable JSON shape.
- [x] Implement strict manifest and deterministic fact helpers without ambient global state.
- [x] Run protocol/repository focused tests and retain the RED/GREEN evidence in WI-38.

### Task 2: Make attach create the minimum idempotent repository scaffold

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-cli/tests/attach.rs`
- Create or modify: `crates/cockpit-repository/tests/repository_context.rs`

**Interfaces:**
- `attach` creates `.ai/cockpit.toml`, `.ai/project.json`, `.ai/agent-interface.json`, active/archive Work Item directories, evidence, decisions, and knowledge directories.
- Repeated attach preserves stable IDs and manifest bytes.

- [x] Add tests for manifest creation, byte-stable repeated attach, absence of provider/global files, and parallel A/B attach isolation.
- [x] Implement idempotent manifest generation using the existing atomic write boundary.
- [x] Verify two repositories can attach concurrently without sharing IDs or files.

### Task 3: Add the shared Work Item scaffold API and CLI command

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-cli/src/main.rs`
- Create: `crates/cockpit-cli/tests/work_item_scaffold.rs`
- Modify: `crates/cockpit-protocol/src/lib.rs` only if optional scaffold metadata needs a typed field

**Interfaces:**
- Add `work-item new --repo <path> --id <id> --mode <mode>`.
- Use a shared `ContractScaffoldOptions`/builder for both `work-item new` and transitional `start`.
- New output reports resolved facts, required human fields, and `state: not_ready`.

- [x] Add failing CLI tests for required explicit repo, four auto-filled facts, empty/unknown human fields, validator readability, and no positive states.
- [x] Implement the scaffold through one underlying API; keep `start` compatibility and avoid duplicate generation logic.
- [x] Add a parallel A/B Work Item test that proves Contract and profile/snapshot identity isolation.

### Task 4: Add read-only candidate profile proposal

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-repository/src/lib.rs` only for a pure proposal adapter if needed
- Create: `crates/cockpit-cli/tests/profile_propose.rs`

**Interfaces:**
- Add `profile propose --repo <path>` using existing observation/evolution logic.
- Emit a candidate/proposed amendment without modifying `.ai/project.json` bytes or digest.

- [x] Add a test that snapshots baseline bytes/digest, runs propose, and proves both remain unchanged.
- [x] Add candidate/proposed state assertions and ensure no apply path is implied.
- [x] Run CLI profile tests and inspect output as an Agent-facing record.

### Task 5: Synchronize trilingual user documentation

**Files:**
- Modify: `README.md`, `README.zh-CN.md`, `README.ja.md`
- Modify: `docs/capabilities.md`, `docs/capabilities.zh-CN.md`, `docs/capabilities.ja.md`
- Modify: `docs/reference/commands.md`, `docs/reference/commands.zh-CN.md`, `docs/reference/commands.ja.md`
- Modify: `docs/reference/configuration.md`, `docs/reference/configuration.zh-CN.md`, `docs/reference/configuration.ja.md`
- Modify: `docs/architecture.md`, `docs/architecture.zh-CN.md`, `docs/architecture.ja.md` if needed for the scaffold boundary
- Create/update: `docs/work-items/WI-38.md`, `docs/work-items/WI-38.zh-CN.md`, `docs/work-items/WI-38.ja.md`

**Interfaces:**
- Explain one shared Runtime, isolated Repository Contexts, the minimum attach tree, Work Item scaffold output, and the explicit human-input boundary.
- Keep provider install/repair/detach clearly planned and separate from attach.

- [x] Add reader-first quick starts and the exact known-facts/human-input output in all three languages.
- [x] Update command/configuration tables without claiming unimplemented provider adapters.
- [x] Run heading, link, code-fence, terminology, and trilingual parity checks.

### Task 6: Full verification and independent review

**Files:**
- Modify only where verification finds a concrete defect.

- [x] Run `cargo fmt --all -- --check`, package tests serially, full workspace tests, and Clippy with `-D warnings`.
- [x] Run `git diff --check` and the repository documentation review.
- [ ] Run Windows compile/runtime checks and hosted CI for the exact commit when available.
- [x] Re-audit all twelve WI-38 acceptance criteria and record evidence in the three Work Item documents.

## Commit Sequence

1. `feat: add strict repository scaffolding contracts`
2. `feat: add attach and work item scaffolding`
3. `feat: add read-only profile proposals`
4. `docs: document governance scaffolding boundaries`
5. `test: verify repository isolation and final acceptance`

## Self-Review Checklist

- No generated file claims approval, verification, completion, or pass.
- No command can infer a repository from process state.
- Attach does not write provider/global Agent configuration.
- `start` and `work-item new` share code rather than drifting generators.
- Formal profile bytes/digest remain unchanged after proposal.
- A/B parallel tests inspect every repository-owned path, not only IDs.

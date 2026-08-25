# Capability and Profile Projection Implementation Plan

> **For agentic workers:** Execute this plan task-by-task in the dedicated WI-276 worktree. Keep the Contract and all generated evidence repository-bound.

**Goal:** Add strict Rust-native repository capability/profile declarations and bind them to preflight and capability projections without copying the reference runtime.

**Architecture:** Keep the external Runtime request-scoped. Put declaration parsing and identity/digest validation in a focused `project_governance` repository module; expose typed protocol records and add the projection as an optional field on the existing capability registry. Preflight consumes only explicit declarations and adds stable unknowns for missing or contradictory inputs.

**Tech Stack:** Rust workspace, serde JSON, existing `cockpit-protocol`, `cockpit-repository`, CLI/MCP projections, shell/Python conformance gates, Markdown documentation.

**Spec:** `docs/superpowers/specs/2026-08-25-capability-profile-projection-design.md`

## Global Constraints

- Every repository-bound command uses `--repo /private/tmp/ai-cockpit-wi276-capability-profile`.
- No reference Python/Make/runtime files, static installed manifest, global Agent/MCP configuration, or architecture decomposition.
- New declaration reads are regular-file-only and fail closed on symlink, malformed JSON, unknown fields, foreign identity, stale digest, and duplicate keys.
- Project-level success criteria never replace Contract acceptance or authorize a Work Item.
- Use TDD: each production behavior starts with a failing focused test.
- Preserve user work in the root worktree; only the dedicated worktree is writable.

### Task 1: Typed declaration protocol

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs`
- Test: `crates/cockpit-protocol/tests/intelligence_schema.rs`

- [ ] Add strict `ProjectCapabilityDeclaration`, `ProjectSuccessCriteriaDeclaration`, `ProjectProfilePolicy`, and `ProjectGovernanceProjection` types with explicit schema versions, repository identity, reviewed snapshot binding, and semantic output digests.
- [ ] Add failing tests for valid round-trip, unknown-field rejection, wrong type rejection, and duplicate-key rejection.
- [ ] Run the focused protocol test and observe the intended failures.
- [ ] Implement the minimal types and serde defaults required for existing protocol-v1 records to remain readable.
- [ ] Run the focused protocol test to green.

### Task 2: Repository declaration reader and dynamic surface

**Files:**
- Create: `crates/cockpit-repository/src/project_governance.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Test: `crates/cockpit-repository/tests/project_governance.rs`

- [ ] Write failing tests for valid declarations, missing declarations, malformed JSON, unknown fields, symlink paths, foreign repository IDs, stale declaration digests, and two-repository isolation.
- [ ] Write failing tests proving repeated projection leaves file/metadata manifests unchanged.
- [ ] Implement regular-file reads, canonical identity checks, canonical JSON digest calculation, stable unknown codes, and a `ProjectGovernanceProjection` result.
- [ ] Add the projection to `CapabilityTruthRegistry` as an additive optional field and preserve existing output semantics.
- [ ] Run the focused repository tests and then the existing intelligence/status tests.

### Task 3: Preflight capability binding

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-repository/src/governance_controls.rs` if a small pure validator is needed
- Test: `crates/cockpit-repository/tests/contract_preflight.rs`

- [ ] Add failing tests for explicit operation mapping success, missing mapping, insufficient capability, conflict, malformed declaration, and foreign declaration.
- [ ] Add a regression proving a legacy Contract without `operation/requestedOperation` remains compatible and does not require the optional declaration.
- [ ] Add a regression proving intent prose cannot satisfy a missing capability mapping.
- [ ] Implement a pure binding helper that adds stable unknowns/review state before the existing governance evaluator; do not infer or mutate declarations.
- [ ] Run focused preflight tests and confirm missing/contradictory inputs are non-green.

### Task 4: Success criteria and CLI/MCP projection

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-mcp/src/lib.rs`
- Test: `crates/cockpit-cli/tests/intelligence.rs`
- Test: `crates/cockpit-mcp/tests/rpc.rs`

- [ ] Add failing tests for `capability show`/`capability_show` identity, declaration/profile digests, and success-criteria visibility without approval.
- [ ] Implement the same repository function for CLI and MCP so their machine JSON is identical.
- [ ] Preserve human Outcome localization boundaries; do not translate Contract or criteria text.
- [ ] Run focused CLI/MCP tests and read-only manifest checks.

### Task 5: Current repository declarations, inventory, and docs

**Files:**
- Create: `.ai/project/capabilities.json`
- Create: `.ai/project/success_criteria.json`
- Create: `.ai/project/profile-policy.json`
- Modify: `tests/conformance/reference_file_inventory.py`
- Modify: `tests/conformance/reference_file_inventory.json`
- Modify: `tests/conformance/reference_file_inventory_test.sh`
- Modify: tri-language capabilities, commands, configuration, and reference-file-comparison docs

- [ ] Add only explicit human-owned declarations for this repository; do not populate unsupported claims or approval fields.
- [ ] Change each of the four scoped inventory records from `migrate-gap` only when the new Rust-native counterpart and tests prove the corresponding responsibility; retain explicit External boundary wording for the static installed-surface manifest.
- [ ] Add regression checks for all six capability-status records and the four former gaps.
- [ ] Update all three languages with the same boundary, commands, unknown behavior, and no-copy rule.
- [ ] Run documentation and conformance acceptance scripts.

### Task 6: Full verification and lifecycle

**Files:**
- Modify: WI-276 active Summary only through Runtime commands

- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run targeted protocol/repository/CLI/MCP tests.
- [ ] Run `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`.
- [ ] Run `cargo test --locked --workspace` in a single process.
- [ ] Run conformance and documentation acceptance scripts.
- [ ] Record checkpoint, verify, finish, archive, and visible human Outcome using installed Runtime `0.2.31`.
- [ ] Push the branch, wait for hosted checks, merge the reviewed PR, run finalization verification, close with structured decision, promote docs, and remove only exact WI-276 resources.

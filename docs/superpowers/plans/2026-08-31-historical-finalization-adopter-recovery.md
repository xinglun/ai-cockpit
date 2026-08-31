# Historical Finalization Adopter Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints.

**Goal:** Make legacy finalization recovery discoverable and auditable without weakening current Runtime identity gates, then prove it with public Release acceptance.

**Architecture:** Add typed read-only historical inventory/projection shared by status and migration planning, a CLI recovery-plan writer-free boundary, and an actionable stale-finalization diagnostic. Keep `finalize-recovery --input` and complete direct-merge `finalize` as the only mutation paths. Extend repository and release tests with real Git fixtures and immutable-byte assertions.

**Tech Stack:** Rust workspace (`cockpit-protocol`, `cockpit-repository`, `cockpit-cli`), serde JSON, Git fixture tests, POSIX release acceptance shell scripts.

**Spec:** `docs/superpowers/specs/2026-08-31-historical-finalization-adopter-recovery-design.md`

## Global Constraints

- Current active/pre-close receipts must still match the executing Runtime identity exactly.
- Historical recovery is `historical_low`, append-only, repository-bound, and never rewrites predecessor bytes.
- `direct_merge_no_pr` requires `pullRequest.number=0`, real merge commit and parents, base, repository identity, and no fabricated PR.
- All repository commands carry explicit `--repo`; no process-global project state or global Agent/MCP writes.
- Release acceptance uses only immutable public artifacts, no source or workspace-binary fallback, and proves cleanup.
- New fields are optional/defaulted where old JSON readers must remain compatible.

### Task 1: Establish failing repository regression tests

**Files:**
- Modify: `crates/cockpit-repository/tests/resource_finalization_transition.rs`
- Create: `crates/cockpit-repository/tests/historical_finalization_inventory.rs`

**Interfaces:**
- Consumes: current `record_resource_finalization`, `verify_resource_finalization`, and close helpers.
- Produces: failing tests for stale closed projection, pending recovery diagnostics, inventory facts, and immutable predecessor bytes.

- [ ] **Step 1: Add a stale closed receipt projection test**

Create a real archive fixture with an older receipt Runtime and a valid close
binding; assert current Runtime verification returns `historical_verified`,
`historical_low`, and the original receipt digest.

- [ ] **Step 2: Add a pending stale receipt diagnostic test**

Create the same older receipt without a close decision; assert verification is
nonzero, includes the stable recovery action/plan command, and leaves all
predecessor bytes unchanged.

- [ ] **Step 3: Add inventory mutation tests**

Cover shared-primary retained, direct-merge no-PR, missing/zero PR number,
foreign repository identity, stale Contract base, malformed JSON, symlink, and
forged Git parent cases.  Assert invalid entries never become green.

- [ ] **Step 4: Run the focused tests and confirm RED**

Run `cargo test --locked -p cockpit-repository --test resource_finalization_transition --test historical_finalization_inventory -- --test-threads=1` and record the expected missing projection/diagnostic failures before production edits.

### Task 2: Add typed historical inventory and migration/status projection

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-repository/tests/historical_finalization_inventory.rs`

**Interfaces:**
- Consumes: archive/finalization/close readers, `repository_id`, Contract base, current Runtime.
- Produces: `HistoricalFinalizationInventoryItem`, `historical_finalization_inventory`, optional `MigrationPlan.historical_finalization`, and `RepositoryReadiness.historical_finalization`.

- [ ] **Step 1: Define the typed optional output fields**

Add serde-defaulted types with stable states (`historical_verified`,
`recovery_required`, `invalid`), assurance, predecessor digest/path, known
facts, and safe actions.  Keep existing fields and migration state unchanged.

- [ ] **Step 2: Implement deterministic inventory scanning**

Enumerate archived Work Items in stable order, accept only regular
non-symlink receipts, compute canonical predecessor digests, validate identity
and Contract base, and inspect close binding without rewriting any file.

- [ ] **Step 3: Wire inventory into status and migrate plan**

Expose the same vector in both read-only outputs.  `migrationType=none` remains
correct for schema-current repositories; historical actions are separate from
schema migration safe actions.

- [ ] **Step 4: Run focused tests and review output stability**

Run the inventory test and existing CLI attach/status tests; assert old fields
remain unchanged and the new collection is deterministic.

### Task 3: Add read-only recovery-plan CLI and actionable verification

**Files:**
- Modify: `crates/cockpit-cli/src/main.rs`
- Modify: `crates/cockpit-repository/src/lib.rs`
- Modify: `crates/cockpit-repository/tests/resource_finalization_transition.rs`
- Create: `crates/cockpit-cli/tests/historical_recovery_plan.rs`

**Interfaces:**
- Consumes: inventory and Git facts.
- Produces: `work-item finalize-recovery-plan` JSON; stale `finalize-verify` stable recovery diagnostics; no-write guarantees.

- [ ] **Step 1: Add failing CLI plan tests**

Assert the command emits `knownFacts`, `suggestedInput`, and
`humanInputRequired`, handles shared retained and direct merge hints, and does
not create `.ai/decisions` files.

- [ ] **Step 2: Implement the read-only plan function and command**

Derive repository/Work Item/base/predecessor/runtime facts and Git merge
parents where determinable.  Keep actor, authority, reason, timestamp, and
unknown fields explicit rather than inventing them.

- [ ] **Step 3: Implement closed historical projection and pending guidance**

Validate close binding for closed stale receipts and return low-assurance
projection.  For pending stale receipts, retain fail-closed behavior but add a
stable recovery code and exact plan/finalize command to the error.

- [ ] **Step 4: Run CLI and lifecycle focused tests**

Run `cargo test --locked -p cockpit-cli --test historical_recovery_plan -- --test-threads=1` and the repository finalization tests; assert current receipts still reject foreign Runtime identity.

### Task 4: Extend documentation and public Release acceptance

**Files:**
- Modify: `docs/reference/commands.md`
- Modify: `docs/reference/commands.zh-CN.md`
- Modify: `docs/reference/commands.ja.md`
- Modify: `docs/reference/repository-workflow.md`
- Modify: `docs/reference/repository-workflow.zh-CN.md`
- Modify: `docs/reference/repository-workflow.ja.md`
- Modify: `tests/release/adopter_acceptance.sh`
- Modify: `tests/release/adopter_acceptance_test.sh`
- Modify: `tests/release/adopter_upgrade_acceptance.sh`
- Modify: `tests/release/adopter_upgrade_acceptance_test.sh`

**Interfaces:**
- Consumes: final CLI behavior and typed output.
- Produces: synchronized three-language recovery instructions and immutable public-artifact acceptance evidence.

- [ ] **Step 1: Add documentation regression assertions**

Require all languages to document the distinction between schema migration,
historical recovery, `finalize-recovery-plan`, shared retained, and complete
direct-merge receipts.

- [ ] **Step 2: Add public-binary historical acceptance lane**

Extend the isolated release harness with a deterministic legacy fixture, public
binary download/checksum binding, plan/output assertions, tamper negatives, and
cleanup manifests. Never call `cargo build`, `cargo run`, or a workspace binary.

- [ ] **Step 3: Run shell wrapper tests**

Run `bash tests/release/adopter_acceptance_test.sh` and
`bash tests/release/adopter_upgrade_acceptance_test.sh`; assert invalid
artifacts fail and every temporary run root is cleaned.

### Task 5: Full verification and release preparation

**Files:**
- Modify: `docs/work-items/WI-449-historical-finalization-adopter-recovery.md`
- Modify: `docs/work-items/WI-449-historical-finalization-adopter-recovery.zh-CN.md`
- Modify: `docs/work-items/WI-449-historical-finalization-adopter-recovery.ja.md`

**Interfaces:**
- Consumes: all implementation evidence and public acceptance receipts.
- Produces: complete tri-language Work Item record ready for finish/archive/PR.

- [ ] **Step 1: Run full Rust quality**

Run `cargo fmt --all -- --check`, `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`, `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`, and `git diff --check`.

- [ ] **Step 2: Run documentation/conformance/release policy checks**

Run the repository's current `tests/docs`, `tests/conformance`, and release
policy scripts declared by the final Contract; record exact outputs in Summary.

- [ ] **Step 3: Verify the visible Outcome and archive evidence**

Run the Runtime finish/archive path, confirm the human Outcome contains marker,
status, unknowns, evidence, decision, impact, and next action, then verify the
archive manifest and all digests.

- [ ] **Step 4: Commit the reviewed Work Item branch**

Use a Japanese commit message after all checks pass; do not commit to local
main or delete the branch before PR review.

# Task Outcome Event Parity Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task with review checkpoints.

**Goal:** Align the Rust Task Outcome event stream with the pinned reference semantics for event families, finding deduplication, correction/supersession, and deterministic generation without copying the reference Python/Make implementation.

**Architecture:** Keep `TaskOutcomeEvent` as a strict Rust-native JSONL record bound to one repository and Work Item. Add an optional fingerprint and explicit event-family allowlist, validate append-only relationships and deduplication in one repository validator, and generate events from already typed report sections. Preserve historical schema-1 event streams and keep provider publication, profile projection, and inventory bookkeeping outside this Work Item.

**Tech Stack:** Rust workspace (`cockpit-protocol`, `cockpit-repository`), serde JSON, Rust integration tests, Markdown documentation.

**Spec:** `/Users/sei-rinn/dev/workspace_python/ai-cockpit-template/docs/maintainers/task-outcome-events.md` at pinned commit `fde3380f81fea5fd2e288f7a8849f737dc074060`.

## Global Constraints

- The Rust wire shape remains semantic parity, not byte-for-byte compatibility with the reference template.
- Every event remains repository- and Work Item-bound; events never grant lifecycle, merge, release, provider, or enterprise authority.
- Historical valid schema-1 streams remain readable and are never rewritten.
- Corrections and post-fix recurrence are append-only; no event line or archive byte may be edited in place.
- Finding fingerprints are deterministic and must not include secrets or absolute local paths.
- Contract source text remains in its original language; localized labels are presentation only.
- No changes to the reference inventory, parity ledger, CI/release workflows, global Agent/MCP configuration, or object repositories.

### Task 1: Extend the strict event model and validator

**Files:**
- Modify: `crates/cockpit-protocol/src/lib.rs` (`TaskOutcomeEvent`)
- Modify: `crates/cockpit-repository/src/lib.rs` (`validate_task_outcome_events` and event helpers)
- Test: `crates/cockpit-repository/tests/task_outcome_events.rs`

**Interfaces:**
- `TaskOutcomeEvent.finding_fingerprint: Option<String>` serializes as `findingFingerprint` and is optional for historical events.
- `validate_task_outcome_events` accepts the reference event families (`finding`, `risk`, `warning`, `confirmation`, `stop`, `resume`, `resolution`, `risk-accepted`, `check-pass-after-fix`, `prevention`, `completed`, `cancelled`, `event_corrected`, `event_superseded`) plus existing `blocked` and `recovered` compatibility events.
- A fingerprint is required for `finding` and `risk` events, repository-relative and non-empty; duplicate fingerprints are rejected unless the later event is explicitly marked as post-fix recurrence through `event_corrected`/`event_superseded` relationship metadata.

- [ ] **Step 1: Add failing tests** for each accepted event family, required finding fingerprint, duplicate fingerprint rejection, and correction/supersession ordering.
- [ ] **Step 2: Run the focused test** with `cargo test --locked -p cockpit-repository --test task_outcome_events`; it must fail before implementation.
- [ ] **Step 3: Add the optional typed fingerprint field** with `serde(default, skip_serializing_if = "Option::is_none")` and extend the validator's explicit event-family set and fingerprint/relationship checks.
- [ ] **Step 4: Re-run the focused test** and existing recovery/archive event tests; all must pass.

### Task 2: Generate events for typed report sections with deterministic deduplication

**Files:**
- Modify: `crates/cockpit-repository/src/lib.rs` (`append_task_outcome_events`)
- Modify: `crates/cockpit-repository/tests/task_outcome_events.rs`

**Interfaces:**
- `append_task_outcome_events` emits one event per non-empty finding, risk, warning, forced stop, resolution, recurrence-prevention, avoided-impact, residual-risk, and completion claim using the report's evidence references.
- `findingFingerprint` is `sha256(event family + normalized claim text + sorted evidence references)` and is stable across repeated generation.
- Re-running generation with the same report is idempotent; a changed finding after verification is represented by a new correction/supersession-linked event rather than replacing the old event.

- [ ] **Step 1: Add failing tests** that construct report sections, assert event-family coverage/fingerprints, and verify repeated generation does not duplicate unchanged findings.
- [ ] **Step 2: Run the focused test** and capture the expected missing-event failure.
- [ ] **Step 3: Implement deterministic normalization/fingerprint and section-to-event mapping**, keeping avoided impact conditional on report claim provenance and never inventing a benefit or authority.
- [ ] **Step 4: Re-run focused report/event tests** and verify archived event bytes/digests remain bound.

### Task 3: Document semantic parity and non-goals in three languages

**Files:**
- Modify: `docs/reference/task-outcome-events.md`
- Modify: `docs/reference/task-outcome-events.zh-CN.md`
- Modify: `docs/reference/task-outcome-events.ja.md`
- Modify: `docs/features/task-outcome-report.md`
- Modify: `docs/features/task-outcome-report.zh-CN.md`
- Modify: `docs/features/task-outcome-report.ja.md`

**Interfaces:**
- Each language documents the supported event families, `findingFingerprint` deduplication, correction/supersession and post-fix recurrence, relationship ordering, privacy/no-score rules, conditional avoided impact, and immutable archive behavior.
- Each language explicitly states that Rust semantic parity is not source JSON wire compatibility and that provider publication evidence, project-profile locale projection, and status/PR projection remain separate boundaries.

- [ ] **Step 1: Update the English reference docs** with the exact Rust behavior and non-goals.
- [ ] **Step 2: Translate the same claims into Chinese and Japanese** without changing protocol names or governance meaning.
- [ ] **Step 3: Run documentation acceptance and three-language counterpart checks.**

### Task 4: Verify, finalize, and hand off

**Files:**
- Modify: `.ai/work-items/active/WI-457-task-outcome-event-parity.summary.json` (Runtime-generated only)
- Modify: `.ai/work-items/active/WI-457-task-outcome-event-parity.*` (Runtime-generated lifecycle artifacts only)

- [ ] **Step 1: Run `cargo fmt --all -- --check`, focused tests, and `cargo test --locked --workspace`.**
- [ ] **Step 2: Run the Contract's documentation checks and inspect the diff for scope violations.**
- [ ] **Step 3: Execute `checkpoint`, `verify`, and `finish` with the installed Runtime and record all unknowns honestly.**
- [ ] **Step 4: Archive, push, wait for hosted CI, merge the reviewed PR, then close and run `python3 tests/docs/promote_closed_work_item.py --repo <repository> --check-all`.**
- [ ] **Step 5: Remove only the exact merged branch/worktree after `ready_on_base` is proven; update the next comparison ledger in a later non-overlapping Work Item after WI-445 has closed.**

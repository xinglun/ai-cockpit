# WI-1089 preserved canonical recovery implementation plan

> **For agentic workers:** Execute serially in the existing WI1089 branch/PR after actual native amended Contract review. Use superpowers:executing-plans when available. No subagents or parallel WI1090 source work.

**Goal:** Recover a fully proven amendment-invalidated historical verification path through exact material record, four actual checks and fixture finish/archive, preserving failed history.

**Architecture:** A sealed predecessor proof comes from validated native amendments. A narrow same-byte historical receipt validator feeds the bounded recovery action predicate; ordinary current-evidence validity remains unchanged.

**Tech Stack:** Existing Rust repository services, CLI/MCP, native typed receipts and bounded nofollow I/O.

**Spec:** bounded-repair-design-and-acceptance.proposal-only.json; snapshot-comparison.reported-facts.agent-recorded.json.

## Global constraints

- One WI1089 branch codex/wi1089-blocked-outcome-material-review-recovery, one PR1048; append normal commits, no c960 rewrite/force push.
- Only three implementation files: src/contract_amendment.rs, src/lib.rs, src/status_projection.rs in cockpit-repository. Tests: repository/tests/status_projection.rs and CLI/tests/contract_amendment_processes.rs. Existing six governance docs remain scoped.
- Preserve external c960/source+working-governance snapshot, combined721129 and paused WI1090. No real Sentinel, actual WI finish/archive, merge, Issue close or cleanup.
- Baseline retained-canonical regression must fail on unchanged c960 implementation before fixing. Real consumer raw inputs are unavailable: label equivalent fixture and missing fields.
- Single worker; bounded commands at most900 seconds; isolated target; CARGO_INCREMENTAL=0; no unchanged-source CI/build repeats.
- New Contract confirmation and future exact material residual-risk acceptance are separate. Do not change materialInspectionReview profile/authoritySource in this amendment.

## Review focus

1. Missing fixture hiding retained canonical: Task1 creates old typed success through actual Native operations before blocked Outcome and amendment.
2. Stale before nested integrity validation: Task2 validates the same raw bytes fully before granting finite new-evidence production.
3. Forged Summary markers or foreign predecessor: Task2 seals only native full-chain proof and Task3 mutates one class at a time.
4. Historical/current snapshot conflation: Task2 separates historical plan binding and current-source freshness; no unverified HEAD/root/equality guard.
5. Passing an action query without terminal execution: Task3 asserts four durable execution results, fresh evidence and actual fixture finish/archive.

## Task1: retained-history baseline before implementation

**Files:** repository/tests/status_projection.rs; CLI/tests/contract_amendment_processes.rs.
**Interfaces:** Existing native start/preflight/controls/checkpoint/verify/finish/amend/material APIs. No manually forged canonical success, Outcome or native amendment.

- [ ] Add retained_canonical_amendment_recovery_fixture: generate an actual typed old successful canonical receipt through Native execution; retain an independently hashed read-only input copy; create a real blocked Outcome using an actual rejected lifecycle operation, then sensitive amend and fresh fixture identity-bound review.
- [ ] Record fixture inventory: old canonical present, schema/capture mode/typed receipt/Contract/snapshot/producer/plan/coverage and old Outcome/transition anchors; declare differences from the unpublished actual consumer JSON.
- [ ] Add retained_canonical_after_amendment_reaches_exact_material_action. Run targeted test against unchanged c960 implementation (or saved c960 candidate binary); capture the expected action rejection without editing implementation. Keep the Missing baseline.
- [ ] Stop for diagnosis if the fixture fails at a different earlier gate; do not weaken the fixture or change a guard merely to reach the expected line.

## Task2: trusted history and same-byte validation

**Files:** repository/src/contract_amendment.rs; repository/src/lib.rs; repository/src/status_projection.rs.
**Interfaces:** Proposed sealed ValidatedAmendmentPredecessor (private constructor), narrow validated historical evidence proof, existing status/action-admission entrypoints. No protocol/schema/action addition.

- [ ] Expose a crate-private sealed proof only from fully validated prepared/committed history and checkpoint/revalidation anchors. Bind the old receipt Contract to a unique predecessor of current Contract; reject pending/forked/tampered chains.
- [ ] Read old canonical through bounded nofollow I/O once; reject duplicate keys and validate the same bytes for digest and semantic meaning. Reuse full strict typed receipt/plan/execution/retention checks with the proof's explicit historical Contract digest; never let contract_digest_for_evidence silently reread current Contract for historical validation.
- [ ] Validate supported producer identity/format and historical snapshot/plan self-bindings separately from current clean-source and execution Runtime authority. Equal old/current b46 is reported data for the known case, not a substitute for latest raw evidence; unsupported relations produce a precise bounded diagnostic or supported recovery, not generic Stale acceptance.
- [ ] Replace only the Missing-only recovery prerequisite with Missing OR sealed, fully proven amendment-invalidated history. Preserve current stale/invalid status, old red, current review/material/no-Finding/clean-source and retry restrictions; actual services recompute current admission.
- [ ] Run the same Task1 regression to green after the implementation change. No source broadening or universal Invalid whitelist.

## Task3: full lifecycle and representative negatives

**Files:** Same two test files and existing six governance documents.
**Interfaces:** Real CLI and MCP adapters, current Native plan/verify, receipt execution records, fixture finish/archive.

- [ ] Extend the retained-history fixture through current exact independent material decision, plan-only(processes0), actual verification of all four declared fixture commands, current receipt/preflight and fixture finish/archive. Assert nodesExecuted4, nodesReused0, maxConcurrentProcesses1, matching plan/results/execution records and all exit statuses passed. A single true process cannot satisfy acceptance.
- [ ] Compare the retained original snapshot and old event/receipt hashes, and separately inspect current/archived projection bytes. No receipt deletion, reset, Summary edit or history rewrite to unlock.
- [ ] Mutate representative classes: malformed/duplicate/symlink/digest error, foreign repo/WI/nonpredecessor, wrong marker/transition, unsupported producer, Finding, dirty/unavailable source, stale request and missing/stale/foreign authority. Each precondition rejection starts zero command processes and publishes no success receipt.
- [ ] Exercise an actual required command failure and existing exact retry. Failed executions may spawn but cannot yield successful completion or finish/archive; wrong retry remains rejected.
- [ ] Cover current valid and Missing routes without Cartesian expansion. Use existing CLI test helper to assert real MCP/service parity.
- [ ] Update scoped three-language WI and parity docs with actual results/limits; no claims of real Sentinel recovery from equivalent fixtures.
- [ ] After native status/validate and fresh admission, run the current six declared formal nodes serially once: fmt; all-target clippy; repository status_projection; CLI contract_amendment_processes; MCP material_review; repository material_review. Run one final CI only after source settles. No automatic full verification during Contract preparation.
- [ ] Deliver exact final source/runtime/request plus actual complete fixture evidence for local Sentinel revalidation. Original real four commands still require local acceptance; no actual WI1090/WI1089/Sentinel closure authorization is inferred.

## Self-review

All known old-canonical, complete-chain, same-byte, current binding, negative and terminal-transition requirements map to the tasks above. No future human/material decision or test receipt has been invented. If the scoped files cannot implement a required proof safely, report that exact gap before expanding scope.

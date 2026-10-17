# Runtime Activation Continuity and Present Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: use superpowers:executing-plans after actual native admission. Execute serially in this existing isolated worktree; do not spawn subagents or request another planning confirmation.

**Goal:** Preserve strict journal integrity across real forward activation and explicitly reviewed present recovery of eligible legacy gaps.

**Architecture:** The native activation writer produces actual-time A/B/owner/Runtime/journal prepared/committed evidence. Shared repository APIs validate the entire history and prospective transition before publication. A separate present-time recovery plan/record accepts only an exact finite baseline for generating a fresh preflight; missing historical authentication stays unknown.

**Tech Stack:** Existing Rust workspace, serde strict types, Git temporary-repository fixtures, shared CLI/MCP repository APIs. No new external service or authorization database.

**Spec:** `design.v2.json` and `technical-approval-source.json` in this directory; immutable earlier design remains historical.

## Global Constraints

- Current phase permits plan/brief/source preservation, normal commit/push and native amend/request only; no product/test code until the actual native human gate.
- Exactly 12 approved Rust files; six reference documents; own WI governance and three briefs. Reference-parity changes are limited to the WI-1088 row in each language.
- One worker, one heavy build at a time, isolated target; each command node <=900 seconds. No Runtime installation, merge, close, deletion, Sentinel operation or original WI-1087 recovery.
- Do not repeat start or change mode: the actual Contract is already implementation_active / implementation. Use native amendment only; never hand-edit generated records.
- Keep all existing integrity checks and negative tests. Missing, foreign, pending, partial, stale, expanded or unexplained post-checkpoint input fails closed.

## Review Focus

- Wrong incoming request must not finish an unrelated pending transaction: assert bytes and phase unchanged.
- A partially published forward/recovery record cannot admit work: assert no committed proof or business permission.
- B-only human review and rehashed discontinuity must not authenticate A-to-B: retain and extend strict rejection tests.
- CSV changes beyond the exact enumerated legacy pair, or extra plan/authority/actor changes, reject before publication.
- CLI/MCP must share strict request/record semantics, exact tool count and schemas; unsupported surfaces fail closed.

## Files

- `crates/cockpit-repository/src/contract_amendment.rs`: Shared complete integrity/continuity assessment, strict activation evidence and present recovery validation, pre-publication prospective checks and immutable recovery request/record.
- `crates/cockpit-repository/tests/contract_amendment.rs`: Activation boundary and legacy recovery, immutable-history/CAS/pending/foreign-binding negatives, existing rehashed-discontinuous test remains strict.
- `crates/cockpit-cli/tests/contract_amendment_processes.rs`: Actual temporary-repository CLI reproduction first; common CLI/MCP request/record and no-write failure behavior.
- `crates/cockpit-repository/src/lifecycle.rs`: Forward native activation evidence and bounded lock/phase handling; refuse unexplained human-plan changes before writing rather than emitting a fictitious bridge.
- `crates/cockpit-protocol/src/lib.rs`: Standalone strict activation/request/recovery record types and explicit capability discovery; no new Contract root field or real grant.
- `crates/cockpit-repository/src/status_projection.rs`: Expose only the bounded metadata recovery path for precisely eligible gaps while retaining the blocker and withholding business actions. Other invalid cases fail closed.
- `crates/cockpit-repository/src/lib.rs`: Current explicit contract_amendment exports do not expose the proposed common request/record APIs; add only the narrow exports needed by both adapters.
- `crates/cockpit-cli/src/main.rs`: Explicit native request/record command dispatch; no fallback to old start/amend/review commands.
- `crates/cockpit-mcp/src/lib.rs`: Strict schemas and dispatch to the same request/record repository APIs; unsupported older surfaces remain unsupported.
- `crates/cockpit-repository/tests/lifecycle_entry.rs`: Actual forward activation writer, phase failures, missing record, plan-field mismatch, duplicate/pending/CAS and ownership tests.
- `crates/cockpit-mcp/tests/material_review.rs`: Preserve exact MCP tool-count assertion; with two new tools assert 35 and exact recovery names/strict schemas while retaining material review regression.
- `crates/cockpit-mcp/tests/usage.rs`: Preserve exact tools/list count and usage schema regression; assert exact recovery names/schemas, no lower-bound replacement or removed assertions.

## Task 0: Native scope and human gate — current task

- [ ] Preserve exact relayed technical approval, revised finite-field/stage policy and all not-run acceptance plans.
- [ ] Commit/push source and own briefs under the existing planning scope.
- [ ] Check actual drift and Contract digest, then apply one strict native amendment; no start replay.
- [ ] Read the real preflight request and complete binding; preserve outputs, commit/push, re-register actual final HEAD and return the gate. Record no human response.

## Task 1: Real temporary-repository RED — after native human admission

**Files:** CLI process tests and existing temporary repository fixtures.

- [ ] Add the native not_ready -> amendment -> start --prepare -> fixture review/checkpoint -> amendment reproduction inside a disposable Git repository with local bare remote.
- [ ] Run `cargo test --locked -p cockpit-cli --test contract_amendment_processes` against the unmodified implementation and preserve the actual continuity failure, transaction pair counts and raw output.
- [ ] Verify the failure is the identified boundary defect, not setup or permission failure; commit the RED evidence/test stage without claiming the repair passed.

## Task 2: Forward writer and prospective integrity

**Files:** protocol lib, repository lifecycle/amendment/lib and lifecycle/amendment tests.

**Interfaces:** `ActivationTransitionRecord` with actual prepared/committed evidence; existing `activate_not_ready_scaffold` uses the bounded writer. Validate complete history and candidate transition before publication under current ownership/CAS.

- [ ] Add rejection tests for extra plan fields, foreign owner/Runtime/journal, missing proof, post-checkpoint gap, partial publication and wrong-request/unrelated-pending cases; observe RED.
- [ ] Implement only the finite policy in `design.v2.json`: three activation-owned forward fields; other human-plan fields unchanged. Preserve raw evidence and all journal checks.
- [ ] Run `cargo test --locked -p cockpit-repository --test lifecycle_entry` and `cargo test --locked -p cockpit-repository --test contract_amendment`; retain rehashed discontinuity rejection and commit this bounded stage.

## Task 3: Explicit present-time recovery shared by CLI/MCP

**Interfaces:** `ActivationRecoveryRequest`, `ActivationRecoveryDecision`, `ActivationRecoveryRecord`; `plan_activation_recovery(root: &Path, work_item_id: &str, runtime: &RuntimeContext) -> Result<ActivationRecoveryRequest, ObserverError>`; `record_activation_recovery(root: &Path, work_item_id: &str, decision: &ActivationRecoveryDecision, runtime: &RuntimeContext) -> Result<ActivationRecoveryRecord, ObserverError>`.

- [ ] Add RED tests for exact legacy eligibility, missing start/checkpoint bindings, broad CSV exemption, B-only review substitution, extra scope/authority/actor/source changes, stale request/summary/history/registration/CAS and pending/partial publication.
- [ ] Implement strict native plan/record with current-baseline-only semantics, actual human response, exact differences/retained unknowns and atomic currentness checks; status admits bounded metadata recovery only.
- [ ] Add `work-item activation-recovery plan|record` and MCP `work_item_activation_recovery_plan` / `work_item_activation_recovery_record`, dispatching to the same repository APIs.
- [ ] Keep both exact MCP count assertions: if exactly two tools are added to the actual baseline of 33, assert 35; assert exact names and strict required/additionalProperties schemas and retain usage/material-review assertions.
- [ ] Run repository, CLI process and both MCP test targets and commit. A record only enables new preflight generation; it never approves C or business actions.

## Task 4: Documentation and full acceptance

- [ ] Update three-language commands and own briefs; append/update only WI-1088 actual-stage rows in the three reference-parity files. Do not alter other WI conclusions or claim unrun acceptance.
- [ ] Run all declared checks serially with the isolated target; preserve actual formal receipts and CI terminal/current states at the exact HEAD.
- [ ] Update Draft PR 1047 with the actual final change and validation. No merge/close/delete or Runtime delivery/original-WI recovery.

All product acceptance, reproduction and verification remain not_run at this planning revision. Technical scope approval is not a native human preflight approval.

# Archived PR CI Stage Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Keep archived PR Contract checks read-only while preserving fresh package coverage and the active PR formal-receipt lane.

**Architecture:** Select one CI lane from the existing `quality-selection.json.selectionMethod` and verify it agrees with the event and Contract location before expensive work. Only the active lane starts hosted Runtime verification and passes its receipt to the Rust gate and repository package consumer; the archived lane gates the checked-out source directly and runs repository/package gates without hosted receipt variables.

**Tech Stack:** GitHub Actions YAML, Bash, Python regression scripts, existing Rust quality gate.

**Spec:** `.ai/work-items/active/WI-1062-archived-pr-ci-stage.contract.json` (canonical digest `78614d82…`).

## Global Constraints

- Base: `origin/main` at `c026e59cd70a6d2117193fba2aebae07d30c1e74`.
- Scope: `.github/workflows/ci.yml`, `tests/ci/workflow_convergence_test.sh`, `tests/ci/quality_route_test.py`, `tests/ci/resolve_work_item_test.sh`, this plan, and `docs/reference/ci-release-evidence.md` only.
- Do not modify Runtime protocol, archived WI-1060/1061 records, or PR #1012; do not fabricate a hosted receipt or relax required gates.
- No PR merge or branch cleanup is authorized by this plan.

## Stage matrix

| Selection method and event | Hosted producer | Rust Contract gate | Repository gates and coverage |
| --- | --- | --- | --- |
| Active `pull_request_binding` / `pull_request_branch_work_item` PR, or active `merged_pull_request_binding` push | Run with candidate Runtime | Gate isolated verification checkout | Consume matching formal receipt; run required package coverage |
| `archived_contract_read_only` PR | Do not start | Existing read-only archived gate on source checkout | Run fresh required checks/package coverage on current head; no hosted receipt or isolated-checkout cleanup |
| `ordinary_repository_route` PR or push | Do not start | Not applicable | Preserve existing required repository/package coverage |
| Unknown method, wrong event, or Contract path/method mismatch | Do not start | Do not run | Fail before package subprocess or green coverage receipt |

## Review Focus

- Archived Contract still triggers producer: archived fixture must show producer skipped and read-only gate selected.
- Archived gate accidentally receives isolated-worktree path: fixture must assert source checkout path.
- Stale hosted receipt leaks to archived package consumer: fixture must assert empty receipt/orchestration variables and fresh package execution.
- Active PR loses formal receipt: fixture must retain producer/gate/consumer order and hosted receipt binding.
- Unknown method or mismatched Contract path: executable route-binding fixture must reject before package marker creation.

---

### Task 1: Route regression and lane validation

**Files:** Modify `tests/ci/workflow_convergence_test.sh`, `tests/ci/quality_route_test.py`; retain `tests/ci/resolve_work_item_test.sh` as the selector behavior check.

**Interfaces:** Consume `target/quality-selection.json.selectionMethod`, `target/quality-route.json.contractPath/stage`, and GitHub event context. Produce a `quality_route.outputs.lane` value `active`, `archived`, or `ordinary` only for valid combinations.

- [ ] Add an executable fixture that runs the workflow's route-binding shell with active, archived, ordinary, unknown, and mismatched event/path selection data; assert the exact lane or early nonzero result.
- [ ] Run `bash tests/ci/workflow_convergence_test.sh` and `python3 tests/ci/quality_route_test.py`; observe a failure caused by the absent lane validation.
- [ ] Add minimal lane validation/output to `.github/workflows/ci.yml`; rerun both tests to green.
- [ ] Run `bash tests/ci/resolve_work_item_test.sh` to retain selector identity coverage.

### Task 2: Lane-specific producer, gate, and package consumer

**Files:** Modify `.github/workflows/ci.yml`, `tests/ci/workflow_convergence_test.sh`, `tests/ci/quality_route_test.py`, `docs/reference/ci-release-evidence.md`.

**Interfaces:** Consume the admitted `lane` from Task 1; active alone produces `target/hosted-runtime-verification.json`; archived and ordinary must not claim that receipt.

- [ ] Add regression assertions for all three lanes: active producer→isolated Rust gate→formal consumer, archived source Rust gate→fresh repository/package coverage with empty hosted variables, ordinary existing repository/package coverage, plus unknown fail-closed before package marker.
- [ ] Run the focused workflow/route tests and observe RED against the current Contract-path-only conditions.
- [ ] Change only the workflow conditions and bounded checkout/receipt wiring needed to make the cases GREEN; update the CI evidence explanation.
- [ ] Run the focused tests, declared Cargo gate test, and formal Runtime verification. Read each result, then request independent code review and exact-head PR CI before any merge decision.

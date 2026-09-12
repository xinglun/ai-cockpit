# CI Archive Route Recovery Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make CI honor the documented `finish/archive → push → reviewed PR` lifecycle without treating an absent active Contract as an error or confusing a main push with Release tag recovery.

**Architecture:** Keep the Runtime Contract gate and Release recovery strict. The shared CI selector will have an explicit ordinary-repository route for pull requests and protected-branch pushes that have no matching active Contract; it will still select exactly one active Contract when present and fail closed on ambiguity. The ordinary route carries no Work Item identity, so downstream CI naturally skips Contract-bound Runtime shadow/gate work while continuing the manifest-backed repository gates.

**Tech Stack:** Bash, jq, Python CI policy tests, Git fixtures, installed AI Cockpit Runtime.

**Spec:** User-provided release/WI optimization addendum, especially lifecycle ordering, early failure, and recovery idempotence requirements.

## Global Constraints

- Do not modify `.ai` generated records except through the Runtime lifecycle.
- Do not accept archived or closed records for explicit immutable-tag recovery.
- Do not weaken active-binding ambiguity, repository identity, base revision, or scope validation.
- Do not invoke Cargo, GitHub Actions, Release publication, or object-engineering checks from the selector regressions.
- Preserve the user's unrelated untracked `.ai/work-items/active/WI-802-release-v0-2-91-recovery.approach.json` in the primary checkout.

### Task 1: Define failing selector regressions

**Files:**
- Modify: `tests/ci/resolve_work_item_test.sh`
- Modify: `tests/ci/workflow_convergence_test.sh`

**Interfaces:**
- Consumes: `tests/ci/resolve_work_item.sh` event and identity inputs.
- Produces: deterministic assertions for ordinary PR/merge selection, active binding, ambiguity, and explicit release recovery.

- [ ] **Step 1: Add the archived post-finish PR case.** Create a fixture with no active Contract, invoke the resolver with `pull_request`, and assert exit zero, `state=ready`, `mode=pull_request`, `selectionMethod=ordinary_repository_route`, and null Contract/Work Item fields.
- [ ] **Step 2: Run the resolver regression to verify the new case fails.** Run `tests/ci/resolve_work_item_test.sh`; expected failure is the current `work_item_contract_unresolved` result for the no-active-Contract PR.
- [ ] **Step 3: Add the ordinary main-push case.** Use a fixture-local fake `gh` executable that returns no merged PRs, invoke the resolver with `push`, `GITHUB_REF=refs/heads/main`, and a full commit head, and assert it succeeds without `to_tag` or `releaseSourceRevision`.
- [ ] **Step 4: Update the convergence policy assertions.** Assert that the resolver contains the explicit ordinary-route branch and that the CI route still keeps `contractPath` empty for that result.

### Task 2: Implement the minimal resolver state machine

**Files:**
- Modify: `tests/ci/resolve_work_item.sh`

**Interfaces:**
- Consumes: PR branch/URL, merged-PR API facts, and explicit recovery inputs.
- Produces: `work_item_selection` receipts with either one validated active Contract or an explicit ordinary-repository route.

- [ ] **Step 1: Separate merge selection from Release selection.** Restrict semantic tag and immutable source-revision validation to `workflow_dispatch` recovery. Make `push` represent the CI merge stage and set its mode to `merge`.
- [ ] **Step 2: Preserve strict active selection.** Keep active Contract scanning, exact branch/PR matching, repository/base/scope validation, and ambiguity failure unchanged for matching active Contracts.
- [ ] **Step 3: Emit the ordinary route only when no active binding exists.** For `pull_request` and `push`, write a ready receipt with `contractPath`, `workItemId`, `baseRevision`, and Contract/source identity fields set to null, plus `selectionMethod=ordinary_repository_route`. Do not create or infer a Work Item identity from archive files.
- [ ] **Step 4: Keep recovery fail-closed.** If `workflow_dispatch` with `publish_existing_tag=true` has no explicit active Contract, retain `work_item_contract_missing`; do not let the ordinary route apply.
- [ ] **Step 5: Run focused regressions.** Run `tests/ci/resolve_work_item_test.sh` and `tests/ci/workflow_convergence_test.sh`; expected result is PASS.

### Task 3: Verify route compatibility and lifecycle evidence

**Files:**
- Modify: `tests/ci/quality_route_test.py` only if the route receipt contract needs an explicit ordinary-route assertion.
- Create: `.ai/evidence/WI-813-ci-archive-route.verification.json` through Runtime verification.

**Interfaces:**
- Consumes: selector receipts and the canonical repository gate manifest.
- Produces: evidence bound to WI-813, the current repository snapshot, current Runtime identity, and passing verification commands.

- [ ] **Step 1: Run the focused shell/Python policy suite.** Run `tests/ci/resolve_work_item_test.sh`, `tests/ci/workflow_convergence_test.sh`, `tests/ci/quality_route_test.py`, `tests/ci/release_gate_policy_test.sh`, and `tests/release/workflow_policy.sh .github/workflows/release.yml`.
- [ ] **Step 2: Run the Contract-declared verification through the installed Runtime.** Use `verify --repo <worktree> --work-item WI-813-ci-archive-route --stage task --command ...` with the exact focused suite, allowing the Runtime to write the verification receipt.
- [ ] **Step 3: Re-run preflight and finish.** Confirm the current Contract, snapshot, Runtime identity, and verification evidence all bind before archiving.
- [ ] **Step 4: Review the generated Outcome.** Use `work-item outcome --repo <worktree> --id WI-813-ci-archive-route --json` and preserve any unknowns rather than inventing a user-visible benefit.

### Task 4: Integrate without repeating expensive release work

**Files:**
- Generated: `.ai/work-items/archive/WI-813-ci-archive-route.*` through Runtime archive.
- Repository: no Release tag or asset changes.

**Interfaces:**
- Consumes: passing focused verification and hosted PR checks.
- Produces: reviewed PR, merged main synchronization, closed WI, and a revalidated PR #790 route without rebuilding or republishing v0.2.91 unless a current identity check proves it is necessary.

- [ ] **Step 1: Run finish and archive through Runtime.** Commit only the planned source/tests/plan and Runtime-generated archive bundle.
- [ ] **Step 2: Push the dedicated branch and inspect the exact hosted route failure/success.** Do not dispatch Release while CI is unresolved.
- [ ] **Step 3: After hosted checks pass, merge only the reviewed PR and synchronize `main`.** Recheck PR #790 using the new base behavior; reuse valid prior build/acceptance evidence where identity bindings permit.
- [ ] **Step 4: Close the Work Item and run `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`.** Report any stale projection as a separate bounded documentation task.


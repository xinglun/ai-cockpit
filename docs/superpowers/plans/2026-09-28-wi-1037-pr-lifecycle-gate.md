# WI-1037 PR Lifecycle Gate Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Admit only the exact PR merge checkout that introduces an ordinary archived Work Item to the temporary `awaiting_merge_close` state, while retaining fail-closed behavior everywhere else.

**Architecture:** Extend the existing narrowly bound merge-transition exception with a pull-request-event counterpart. Bind event number/base/head, merge-ref checkout, its two Git parents, and archive-path introduction; then leave the existing post-merge close requirement unchanged. Update only the three WI-1036 parity rows to reflect the archived-but-not-yet-closed state.

**Tech Stack:** Python 3 standard library, Bash, Git fixtures, Markdown.

**Spec:** `.ai/work-items/active/WI-1037-task8-pr-lifecycle-gate.contract.json`

## Global Constraints

- Preserve the existing default-branch merge-transition exception and all unrelated lifecycle gates.
- Never admit an older unclosed archive, mismatched/malformed PR event, direct push, or later non-merge default-branch commit.
- Do not create a close receipt before integration; formal close remains post-merge.
- Keep changes within the WI-1037 Contract; no Task 9 migration or release/version/tag/publication work.

## Review Focus

- PR event whose base/head disagree with the checked-out merge parents: reject it; cover with a negative real-Git fixture.
- PR event that does not introduce the archived Contract: reject it; cover with a negative fixture.
- Malformed or unavailable event payload and non-PR checkout: retain the existing failure; cover the event-binding boundary.
- Older unclosed archive: remain blocking; retain/extend the `missing-close` assertion.
- Exact merge followed by a later non-merge commit without close: allow only the immediate merge transition; retain the expiry regression.

---

### Task 1: Register the required lifecycle projections

**Files:**
- Modify: `docs/reference/reference-parity.md`
- Modify: `docs/reference/reference-parity.zh-CN.md`
- Modify: `docs/reference/reference-parity.ja.md`
- Create: `docs/work-items/WI-1037-task8-pr-lifecycle-gate.md`
- Create: `docs/work-items/WI-1037-task8-pr-lifecycle-gate.zh-CN.md`
- Create: `docs/work-items/WI-1037-task8-pr-lifecycle-gate.ja.md`

- [ ] **Step 1: Correct the WI-1036 row and add the WI-1037 three-language projection.**

Point WI-1036 at its archived Contract and label it archived but awaiting PR merge and formal close. Register WI-1037 using the established pre-archive status and reserved lifecycle paths; state that verification remains pending.

### Task 2: Bind the pre-merge lifecycle exception to the exact PR event

**Files:**
- Modify: `tests/ci/governance_integrity_gate_test.sh`
- Modify: `tests/ci/governance_integrity_gate.py`

**Interfaces:**
- Consume: `repository_default_branch`, `repository_phase`, `merge_commit_introduces_archived_work_item`, and the existing archived lifecycle branch in `main`.
- Produce: a narrow helper that returns true only when a valid `pull_request` merge-ref event exactly identifies the current merge commit, its base/head parents, the repository default branch, and newly introduced archived Contract.

- [ ] **Step 1: Add the failing positive and negative real-Git PR-merge fixture.**

Create an ordinary archived Work Item without `resourceContext`, then construct base/head commits and an exact two-parent PR merge checkout. Assert that only the archive introduced by a matching event becomes `awaiting_merge_close`; wrong base/head, malformed event, an archive already present in the base, and non-PR context must retain `missing_terminal_decision`.

- [ ] **Step 2: Run the focused gate test and confirm the intended failure.**

Run: `bash tests/ci/governance_integrity_gate_test.sh`

Expected: the exact PR-introduced ordinary archive currently fails with `missing_terminal_decision`; existing positive and fail-closed cases remain otherwise unchanged.

- [ ] **Step 3: Implement the smallest exact-event helper.**

Add the PR counterpart beside `merge_commit_introduces_archived_work_item`; validate event JSON shape and default branch, exact `refs/pull/<number>/merge`, `GITHUB_SHA == HEAD`, exactly two parents matching event base/head SHAs, and an `A` status for the Work Item's archive Contract in the base-to-merge diff. Use it only in the existing no-terminal-decision lifecycle branch, alongside the existing feature/PR and default-branch merge conditions.

- [ ] **Step 4: Re-run the focused gate test.**

Run: `bash tests/ci/governance_integrity_gate_test.sh`

Expected: the exact positive PR case passes; mismatched, malformed, older, direct-push, non-PR, and later-commit cases remain fail-closed.

### Task 3: Verify and close Task 8

**Files:**
- No additional implementation files; use the existing PR and Runtime lifecycle records.

- [ ] **Step 1: Run the declared focused checks and documentation acceptance.**

Run: `bash tests/ci/governance_integrity_gate_test.sh`

Run: `bash tests/docs/documentation_acceptance.sh`

Expected: both exit successfully, including the new real-Git lifecycle assertions and all three parity projections.

- [ ] **Step 2: Refresh Runtime status and require hosted CI on the exact corrective head and PR #997 head.**

Do not bypass required checks. If either exact-head run fails, preserve the evidence and repair only within WI-1037 scope.

- [ ] **Step 3: Integrate the corrective branch into PR #997, then merge PR #997 only after its required checks pass.**

- [ ] **Step 4: Complete the Runtime-authorized archive, synchronized-main verification, and close steps for the Task 8 lineage; clean only exact merged local resources.**

- [ ] **Step 5: Confirm Runtime `readyOnBase` before starting Task 9. Stop before release, tag, or publication and hand off the result for human review.**

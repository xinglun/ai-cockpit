# WI-1039 Task 8 Ordinary Mainline Merge Transition Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enforce WI-1037 A03 for ordinary archived Work Items on the exact default-branch push merge transition without relaxing later or forged states.

**Architecture:** Reuse `merge_commit_introduces_archived_work_item` for the already implemented exact push/HEAD/two-parent/new-contract checks. Add the ordinary non-resource-bound lifecycle branch beside the existing PR and resource-bound paths, and prove it with a real Git fixture.

**Tech Stack:** Python 3 standard library, Bash, Git fixtures, Runtime `0.2.113`.

**Spec:** `.ai/work-items/active/WI-1039-task8-ordinary-merge-transition.contract.json`

## Global Constraints

- Keep changes within the WI-1039 Contract and retain single-WI serial execution.
- Preserve existing PR and resource-bound merge-transition behavior.
- Admit only `GITHUB_EVENT_NAME=push`, the exact configured default branch, exact `GITHUB_SHA == HEAD`, exactly two parents, a newly added archive Contract, and push payload `before == first parent`.
- For push transitions, require the trusted push payload's `before` SHA to equal the merge commit's first parent.
- A later non-merge commit, old archive, malformed identity, or non-default branch remains fail-closed.
- Reuse only complete verification receipts whose Runtime, Contract, and source snapshot identities match.
- Do not start Task 9 or perform a release, tag, version change, or publication.

## Review Focus

- Wrong, missing, or empty advertised SHA must not admit a merge transition; assert the lifecycle finding remains blocking.
- A one-parent or three-parent commit must not be treated as a PR merge; exercise actual Git parents.
- An archive already present in the base must not receive a second merge grace window.
- A stale first parent must not qualify when the push payload's `before` already contains the archive.
- The exact merge commit is temporary only; a following ordinary default-branch commit without close remains blocking.
- Existing ordinary PR merge and resource-bound merge cases must retain their current results.

## File Map

- Modify `tests/ci/governance_integrity_gate.py` to bind the exact-push helper to the push payload's `before` SHA and invoke it for an ordinary archived Work Item.
- Modify `tests/ci/governance_integrity_gate_test.sh` to provide real push payloads, add a non-resource-bound real-Git positive fixture, and cover stale-before fail-closed behavior.
- No production Rust, Runtime, manifest, or user-facing documentation changes are planned; the script is already a registered canonical gate.

---

### Task 1: Add and satisfy the ordinary default-branch transition regression

**Files:**
- Modify: `tests/ci/governance_integrity_gate_test.sh`
- Modify: `tests/ci/governance_integrity_gate.py`

**Interfaces:**
- Consumes: `repository_default_branch`, `merge_commit_introduces_archived_work_item`, and the existing ordinary PR/resource-bound lifecycle branches.
- Produces: an ordinary push-transition predicate that is true only for a newly introduced archive on the exact two-parent default-branch merge commit.

- [x] **Step 1: Add a real-Git ordinary Work Item fixture and assertions.**

  Start from the existing `awaiting-merge-close.json` fixture, remove its resource-context Contract field and resource-context decision before creating the base commit, then create a base that lacks the archived Contract and a feature commit that adds it. Merge the feature commit with `--no-ff` onto `main`.

  Assert that exact push context yields `awaiting_merge_close` with no `missing_terminal_decision`. Also assert that a later one-parent main commit, wrong/missing/empty `GITHUB_SHA`, wrong default branch, one-parent and three-parent integrations, and a case where the archive is already in the base remain blocking. Keep the existing resource-bound fixture unchanged.

- [x] **Step 2: Run the focused gate regression and observe the expected failure.**

  Run: `bash tests/ci/governance_integrity_gate_test.sh`

  Expected before implementation: the ordinary exact merge push is reported as `missing_terminal_decision`; existing resource-bound and PR cases retain their baseline results.

- [x] **Step 3: Implement the minimal ordinary push predicate.**

  Resolve the repository's actual default branch and call `merge_commit_introduces_archived_work_item` only when the archived Contract is ordinary (no resource context and no resource-context receipt). Combine it with, but do not replace, the existing ordinary PR and resource-bound predicates.

- [x] **Step 4: Re-run the regression and manifest tests.**

  Run: `bash tests/ci/governance_integrity_gate_test.sh`

  Run: `python3 tests/ci/repository_gate_manifest_test.py`

  Expected: the ordinary exact merge is `awaiting_merge_close`; malformed identity, old archive, and later direct commit remain blocked; existing paths remain unchanged.

- [x] **Step 5: Confirm Runtime admission and evidence reuse before PR handoff.**

  Inspect Runtime status, validation, and formal receipt bindings. Record only the two focused Contract checks with the candidate Runtime on the exact current snapshot. Reuse the complete main CI receipt for its exact base only; do not launch a duplicate full-workspace command locally.

### Hosted integration boundary

After the exact candidate PR exists, let its required hosted checks execute once on that PR head. Hosted CI is a merge gate, not a reason to repeat local workspace Cargo validation or edit this plan after the receipt is bound.

## Independent review corrective pass

- [x] Add a real Git stale-base case where the push payload's `before` points to a default-branch commit that contains the archived Contract, while the merge's first parent does not; the current gate must fail closed.
- [x] Run the focused gate and observe that this new case fails before the implementation change.
- [x] Require a regular, valid push-event payload and exact `before == first parent` in the shared push-merge helper; leave the existing PR event path unchanged.
- [x] Re-run the full governance-integrity shell regression and the manifest regression.
- [ ] Re-record final Runtime verification on the exact amended Contract and source snapshot; do not run full-workspace Cargo as a substitute or duplicate.

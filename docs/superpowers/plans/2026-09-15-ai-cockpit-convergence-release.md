# AI Cockpit Convergence and Release Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use `superpowers:subagent-driven-development` for independent implementation Work Items and `superpowers:executing-plans` for the serialized integration/release steps. Each implementation Work Item retains its own Contract, branch, worktree, PR, verification, and closure.

**Goal:** Complete the authorized A–E collaboration and performance work, reconcile lifecycle/resource debt without losing branch data, then publish and publicly validate the next eligible AI Cockpit release.

**Architecture:** First repair the current governance blockers and establish one truthful inventory. In parallel, independent agents audit A–E against current main without writes; implementation remains serialized by Contract and shared-file ownership. Cleanup follows verified Work Item closure, and release is a separate final Work Item after the source changes and exact-resource cleanup are complete.

**Tech Stack:** Rust workspace, Python/Shell repository gates, installed `ai-cockpit` Runtime, Git/GitHub Actions, published Release artifacts.

**Spec:** User attachment `AI Cockpit：协作语义修复、观察一致性与性能优化开发指示`; current Work Item Contracts and repository workflow are authoritative for each change.

## Global Constraints

- Keep human authorization, verification, evidence freshness, lifecycle state, and resource cleanup distinct.
- Preserve all historical Contract, evidence, decision, archive, and Release bytes; do not weaken gates or reuse an immutable version tag.
- Never delete a branch/worktree until its exact PR/commit, unique changes, dirty state, and Runtime closure evidence are reconciled.
- Do not run full workspace verification when an earlier deterministic gate can reject the inputs; retain node exit codes, timeout state, and logs.
- Publish only after all A–E acceptance evidence, main CI, required branch/worktree cleanup, actionable GitHub Issues, and release preflight are green. Cleanup and Issue work may interleave when dependencies and write sets permit; release remains last.

---

## Re-baseline checkpoint — 2026-09-15

This is a measured checkpoint, not a completion claim. The remote default is
`origin/main` at `46aabe41cba5dc6af1a7993c7af63a3007b19eef`; Runtime 0.2.92 is
compatible and `doctor` is `ok`. Runtime status in the current W847 worktree
reports 21 active and 692 archived Work Items, with readiness blocked by active
Work Items and a dirty tree. The separate main checkout at
`/Users/sei-rinn/dev/workspace_rust/ai-cockpit` is 78 commits behind and has 8
dirty paths; preserve those records and do not pull over them. The fresh
inventory has 11 non-main remote branches, 17 local `codex/*` branches, and 23
registered worktrees, of which 19 are dirty. The only open PR is #826; the live
open Issue set is #828 and #829.

Cleanup has begun, not completed. WI-845's exact worktree and local/remote
branch were removed after its archive payload was matched to canonical main;
the PR and canonical records remain. Two additional clean detached snapshots
were removed: WI-833's snapshot is an ancestor of merged PR #813, and WI-806's
snapshot is the exact head of merged PR #783 with only an unoccupied zero-byte
lock left in its ignored state. Two local-only snapshots, `codex/wi-803-pre-pr780-merge`
and `codex/wi-806-pre-merge-sync`, were removed after proving each tip is an
ancestor of its corresponding merged PR head (#780 and #783). No remote branch
was deleted in this checkpoint. The root main checkout's eight dirty governance
paths remain untouched.

Main CI run `34914002827` at `46aabe41` is still the latest main result and
failed at `docs_closed_work_item_promotion`,
`docs_work_item_status_consistency`, and `ci_manifest_regression`; dependent
`docs_acceptance` was blocked. The merged PR #826's latest quality run
`34913588118` also failed those three gates while route, Windows, behavioral
oracle, Clippy, format, and workspace package tests passed. No PR CI has been
started for W847 because it has no remote branch/PR and is not Finish-ready.
The user clarified that ordinary PR CI can start as soon as the reviewed PR is
ready; no separate manual CI permission is required. Release dispatch remains
manual and strictly last. The main-gate reproduction established the manifest failure root
cause: its deliberate missing-receipt case runs status consistency first,
which exits on WI-845's pre-policy conditional docs status before receipt
validation. The normalized reproduction matched the recorded diagnostic
digest. Current dirty WI-847 changes contain the policy/gate correction and
focused local tests pass, but the hosted fix remains unverified.

WI-847 is active and checkpointed at the exact `origin/main` base. The fresh
Runtime query at `2026-09-15T06:16:19Z` reports `checkpointed/yellow`,
verification `not_ready`, and `evidence_stale`; do not reuse the earlier state
as current.
The current Contract has six acceptance criteria and zero acceptance evidence
entries. `work-item validate` is blocked by two required scenarios: the full
ordinary no-document lifecycle and independent work-result/cleanup terminal
states. The Contract's `sources` and `verification` arrays are empty. Installed
Runtime 0.2.92's `work-item amend --help` advertises only scope, out-of-scope,
acceptance, and evidence-class additions; the dirty branch contains a proposed
`sourcesAppend`/`verificationAppend` implementation and tests, but that route
has not been exercised through the installed Runtime. The prior formal receipt
does not prove the current acceptance matrix. WI-847 is not finish-ready.

Read-only A–D audits found the following remaining acceptance gaps:

| Track | Proven in source | Not yet proven / defect to resolve |
|---|---|---|
| A — recovery action semantics | Retained evidence and failed-cleanup `finalize-verify` actions are non-destructive. | Missing receipt recommends an unusable recording action; `finalize-verify` classifies absence as corruption. Unknown-cause recovery lacks a deterministic fixture, and current tests assert the defective advice. |
| B — observation consistency | Request-local ledger, content/type/symlink fingerprints, and bounded retry exist. | A mutation window remains after ledger recheck; one CLI lifecycle handoff bypasses guarded assembly; add/remove/identity/finalization mutation and operation-level counter coverage are incomplete. |
| C — real collaboration scenarios | CLI/MCP, three locales, and summary/full surfaces have substantial fixture coverage. | SCN-026–032 expectations are not the production-test oracle; no-history advice is partly hand-assembled; recommended recovery commands are not executed through to resolved state. |
| D/E — measurement then optimization | External CLI samples and some Runtime phase/read counters exist. | Internal metrics are not bound to the same operation/sample; parse and child-process counts and span overlap are missing; P0 does not enforce 100 valid warm samples; diagnostic overhead and the planned cross-scenario WI report are absent. Do not select a hot-path optimization until measurement is valid. |

Branch/worktree audits are still reconciliation inputs, not deletion authority.
Preserve dirty evidence, including WI-801 and WI-832 archive/close bundles,
WI-802's local-only commit and modified outcome, WI-815's unique branch commits
and dirty records, WI-831's untracked Outcome/task reports, the dirty WI-836,
WI-837, and WI-840 governance bundles, and every `observer-snapshot.json`
change. Confirmed safe ref candidates remain distinct from their attached dirty
worktrees. WI-838 provenance and the exact WI-845/WI-846 cleanup binding still
need resolution; open PR #826 remains preserved.

The WI-847 review found three additional in-scope defects: canonical docs say
`finalize` performs provider deletion although the Runtime only records
caller-supplied evidence; pending-close dependency declarations are not
enforced at a lifecycle gate; and Work Item completion has no independent
resource-cleanup terminal domain. The required ordinary no-resource lifecycle
scenario is not exercised end-to-end. These remain WI-847 work; A–C outcome
semantics findings stay in their owning scopes.

The converged order is therefore: finish and review the W847 lifecycle/CI fix;
reconcile the exact dirty lifecycle bundles and classify every resource;
complete A–C against one shared action/observation interface without parallel
writes to those files; establish same-operation D instrumentation before E
optimization; then classify and close or replace each incomplete Work Item
one-for-one, clean only verified exact resources, and confirm main CI. Resource
cleanup and actionable GitHub Issues may be interleaved or parallelized when
their dependencies and write sets permit; a proven Issue prerequisite should
be handled before the exact cleanup it blocks, while unrelated cleanup
continues. Do not let an unresolved dependency create a circular wait or defer
unrelated work. Resolve the complete actionable Issue set and finish exact
resource cleanup before beginning the separately governed public release,
which is strictly the final phase. Any new independent CI root gets its own
bounded Work Item rather than expanding W847 by assumption.

## Fresh inventory checkpoint — 2026-09-15

This later read-only checkpoint supersedes the resource counts above without
rewriting the historical snapshot. Live GitHub now reports 10 non-main remote
`codex/*` branches, 15 local `codex/*` branches, and 17 registered worktrees;
all 17 currently have dirty paths. The main checkout remains 78 commits behind
and its same 8 user-owned governance paths remain untouched. Open PR #826 is
still the only open PR. Open Issues #828 and #829 remain the issue-phase seed
set. No CI was dispatched; latest main run `34914002827` is completed with
failure, and PR #826 remains unmerged.

Since the earlier checkpoint, the following exact resources were reconciled:

- Removed detached WI-833 and WI-806 snapshots after verifying their heads
  against merged PR #813 and #783 respectively.
- Removed local-only WI-803 and WI-806 snapshot branches after confirming
  their tips are ancestors of merged PR heads #780 and #783.
- Removed WI-828 release-source-path worktree and local/remote ref after
  confirming no patch-unique commits remained relative to live main and
  preserving its sole dirty observer snapshot in stash
  `27943c5a28a62d62c103a0ac54d8c2d44cbabc4d`.
- Removed the older WI-831 release-identity worktree and local ref after
  matching its source/docs changes to merged PR #811; preserved the complete
  dirty/untracked snapshot in stash
  `eb83a03c421c1797d810f91c6b94e873bf73cb22`. The final PR #811 branch and
  worktree remain.

The remaining dirty lifecycle and release evidence, open PR #826, and unique
or unresolved refs remain preserved. Cleanup is not complete and no Issue
implementation or release has started. Task 5 remains deliberately between
verified cleanup and release; refresh the live Issue set at its entry.

Independent review has identified a merge blocker in PR #826: its new test
mutates a freshly generated current-format close receipt to `ready` and then
accepts it as historical based on report bindings/digest and timestamp. The
PR does not yet demonstrate an old-Runtime/schema provenance discriminator.
Keep the PR and evidence intact; do not merge until a genuine legacy-format
fixture is bound to a supported source/runtime identity and a current-format
receipt changed to a noncanonical decision is rejected.

The active Contracts also declare the chain WI-802 → WI-803 → WI-804 → WI-805
→ WI-806. Targeted Runtime status reports WI-802 through WI-805 red with
`recovery_decision_invalid`, and WI-806 red with `evidence_contradictory`; the
visible retry receipts describe same-Work-Item retries. Issue #829 currently
names three Sentinel Work Items and requires recovery of archived legacy
close/finalization evidence. The shared “lineage” wording alone does not prove
that #829 is a prerequisite for this Cockpit chain; keep the dependency
unconfirmed until the exact invalid receipt and validator path are matched.

### Task 1: Converge current lifecycle and CI blockers (WI-847)

**Files:** WI-847 Contract scope; `AGENTS.md`; `.ai/README.md`; English/Japanese/Chinese workflow projections; `tests/workflow/resource_finalization_policy*.sh`; `tests/docs/promote_closed_work_item.py`; `tests/docs/work_item_status_consistency.py`; their regression tests; `.ai/project/documentation-policy.json`; CI governance and Rust lifecycle files already declared in WI-847.

**Interfaces:** The Runtime Contract is the write boundary. Promotion and governance gates must consume the same explicit projection rule; resource-bound cleanup is verified before close, while no-resource cleanup follows close.

- [ ] Canonical instructions must say the caller performs provider cleanup; Runtime `finalize` records the bound receipt, and `finalize-verify` validates it.
- [ ] Add an end-to-end ordinary no-resource fixture through start, preflight, checkpoint, verify, finish, archive, and close without tri-language pages or a documentation successor; retain explicit documentation/release route checks.
- [ ] Enforce declared pending-close dependencies at the earliest supported preflight boundary, while unrelated debt remains visible and conflicting scopes fail at start.
- [ ] The resource cleanup-order regression detects contradictory post-close deletion claims in English, Japanese, and Simplified Chinese using real negative fixtures for each language.
- [ ] Project work-result and resource-cleanup states independently; cleanup failure must not rewrite or downgrade verified work evidence.
- [ ] Reproduce the three failed main gates with exact inputs and preserve diagnostics. The manifest root cause is known; the hosted fix remains unverified until the finish-ready reviewed PR is opened and its normal hosted checks reach a terminal result. Ordinary PR CI does not require separate user approval.
- [ ] Investigate the Contract author's missing append path for `sources` and verification commands; satisfy the repository Contract requirements without hand-editing generated records.
- [ ] Implement the smallest shared policy/recovery changes inside WI-847 scope; do not alter historical records.
- [ ] Run the focused gate tests, current Contract preflight, and Runtime formal verification; then complete finish/archive/PR/merge/close through the supported lifecycle.
- [ ] Confirm the merged main CI and WI-847 post-close promotion are green before entering cleanup.

**Verification:**

```bash
bash tests/workflow/resource_finalization_policy_test.sh
python3 tests/docs/promote_closed_work_item.py --repo . --check-all
python3 tests/docs/work_item_status_consistency.py --repo .
python3 tests/ci/repository_gate_manifest_test.py
```

Run the remaining checks declared by the amended WI-847 Contract and its Runtime-generated verification plan.

### Task 2: Audit and finish collaboration semantics (A–C)

**Files:** `crates/cockpit-repository/src/outcome_render.rs`; `crates/cockpit-repository/src/execution_context.rs`; `crates/cockpit-repository/src/lib.rs`; `crates/cockpit-cli/src/main.rs`; `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs`; `crates/cockpit-cli/tests/collaboration_handoff.rs`; `crates/cockpit-repository/tests/scenario_matrix_next_action.rs`; `docs/reference/collaboration-scenario-matrix.json`.

**Interfaces:** CLI and MCP must consume the same typed action projection and request observation context. The scenario matrix is the shared structured oracle, not a second implementation of policy.

- [ ] Map every A, B, and C requirement to current-main code and a named regression; distinguish implemented behavior from missing evidence.
- [ ] Repair any in-scope defect in its existing active Contract. Create a successor only for a different authority/base/scope or immutable failed delivery, with explicit linkage.
- [ ] Cover en/zh/ja and summary/full; exercise the real CLI/MCP path and execute recovery commands in controlled fixtures.
- [ ] Prove mutation, replacement, directory-candidate change, retry, and retry-exhaustion cases cannot return mixed successful facts.

**Verification:** Run the focused Rust integration tests above, followed by each owning Contract's checks and the full workspace gates at final integration.

### Task 3: Audit and complete measurement/optimization (D–E)

**Files:** `crates/cockpit-repository/src/lib.rs`; `crates/cockpit-repository/src/execution_context.rs`; `crates/cockpit-git/src/lib.rs`; CLI/MCP request entry points; `tests/performance/runtime_benchmark.sh`; related performance regression gates and measurement records.

**Interfaces:** A single-operation trace binds operation, scenario, Runtime, and measurement ID. Instrumented counters report actual work; diagnostic overhead is measured separately and never enters evidence identity.

- [ ] Freeze comparable baseline samples before optimization, with source/runtime/toolchain/machine/scenario and raw samples.
- [ ] Make release-grade schema 2 evidence enforce the same minimum of 100 valid warm samples at capture and comparison time; add boundary regressions for 99 rejected and 100 accepted, while keeping percentile-specific reliability reporting explicit.
- [ ] Correct counter and span boundaries; prove counts using known-size files and controlled additional reads.
- [ ] Measure request-local fact reuse and targeted reads before considering persistent caches or parallelism.
- [ ] Implement only optimizations with measured benefit, then rerun identical scenarios and correctness cases; revert any complexity without defensible benefit.
- [ ] Report p50/p95 only when the sample count supports it; list unavailable measurements and reasons.

**Verification:** Run the benchmark and focused counter/trace tests for small/medium/large repositories and representative history/state combinations; run full workspace and repository gates after integration.

### Task 4: Reconcile Work Items, branches, and worktrees

**Files:** Existing `.ai/work-items/active/**`, immutable archived records, exact local/remote branch refs, and their associated worktrees. This is an evidence-preserving cleanup, not a bulk prune.

**Interfaces:** Each item is classified by Contract/receipt, PR state and exact head, unique commits, branch SHA, and worktree cleanliness. Runtime lifecycle commands produce closure evidence; GitHub branch/PR facts are independently observed.

- [ ] Inventory every active WI, remote branch, local branch, open/closed PR, and worktree; record dirty/untracked paths and running processes.
- [ ] Group only disjoint read-only audits in parallel. Serialize shared governance-file updates, Runtime writes, GitHub mutations, and dependent close sequences.
- [ ] For each WI, verify merged/closed/unmerged status, preserve unique commits and dirty data, and use its existing Contract or an authorized linked recovery.
- [ ] If the intended result is already present in main, preserve required audit evidence and discard only the proven duplicate branch/worktree delta; do not create a successor just to close the old record.
- [ ] If a Work Item is genuinely incomplete and cannot safely continue under its existing identity, create exactly one explicitly linked replacement, then close/supersede the predecessor through the supported lifecycle; technical retries stay in the same Work Item, and no replacement chain is created by default.
- [ ] Record any open-Issue prerequisite against the exact dependent Work Item/resource. Execute that bounded Issue Work Item before the dependent cleanup when needed; continue unrelated reconciliation and cleanup, and resume the dependent item only after prerequisite evidence is merged and available.
- [ ] Close only after its route-specific finalization and promotion checks pass; then remove only its exact authorized branch/worktree and prove the postcondition.
- [ ] Recount resources and confirm Runtime `ready_on_base`; retain any resource with unresolved ownership or evidence.

**Verification:** Fresh remote API/ref observations; per-item Runtime status/close receipts; `git status` for every retained or removed worktree; final branch/worktree inventory with zero unclassified entries.

### Task 5: Resolve open GitHub Issues before release

**Entry condition:** Start this phase once the synchronized default branch is
`ready_on_base` for an Issue Work Item; Task 4 need not be globally complete.
Cleanup and Issue Work Items may interleave or run in parallel only when their
Contracts, generated Runtime records, branches, worktrees, and writable paths
are independent. Record exact dependencies; an Issue that blocks a cleanup
item runs first, and that resource remains pending until the merged evidence
is applied. Do not start release while any actionable Issue or exact cleanup
remains unresolved.

**Current live seed set (refresh when Task 5 starts):**

- [Issue #829 — append-only close recovery for an already-selected multi-hop
  successor lineage](https://github.com/xinglun/ai-cockpit/issues/829).
- [Issue #828 — bounded timeout configuration for verification
  commands](https://github.com/xinglun/ai-cockpit/issues/828).

**Execution boundary:** Re-read the live open Issues list when this phase
starts and again before release; do not rely on this checkpoint as current
issue state. Give each independent Issue its own
Work Item/Contract, branch, worktree, and PR. Serialize any Issues whose
Contracts share writable Runtime files; parallelize only after proving
disjoint write sets. Preserve the Issue's acceptance and fail-closed
constraints in its owning Contract. Do not broaden W847 or a cleanup WI to
absorb unrelated Issue scope. Keep an Issue open until its linked implementation
is reviewed, merged, and the Issue acceptance is verified; preserve its history.
Apply any Issue evidence only to the named dependent cleanup item. Complete the
full live Issue set and every authorized cleanup before release preparation.

**Completion gate:** Every Issue open at phase entry is either resolved by a
merged, verified Work Item with direct evidence, or explicitly remains blocked
with its concrete external dependency and user-visible disposition. Do not
silently skip, bulk-close, or treat triage alone as resolution. Recheck the live
open Issues list before entering release preparation.

### Task 6: Publish the next eligible version

**Files:** New release Work Item Contract, release metadata/workflows, release notes, manifests, and the published artifact acceptance records.

**Interfaces:** Version is selected from the highest reserved immutable tag/provider Release state after all source changes and cleanup; public install/upgrade tests consume downloaded Release assets, not local builds.

- [ ] Discover tags and provider Releases independently; choose exactly the next eligible patch and bind source/toolchain/build identity.
- [ ] Run release preflight, full quality/build, candidate fresh-install and N−1 upgrade acceptance, then publish idempotently.
- [ ] Verify public version consistency, downloaded assets, fresh install, and previous-version upgrade in isolated environments.
- [ ] Archive/finalize/finalize-verify/close the release WI and clean only its exact resources.

**Verification:** Successful main CI at the release commit; provider Release and tag identities; manifest/checksum/attestation evidence; public install and N−1 upgrade receipts; final Runtime close and resource inventory.

### Task 7: Independent completion audit

- [ ] For each requirement in the user attachment, identify current implementation and authoritative test/receipt; mark proven, failed, missing, or not applicable with evidence.
- [ ] Confirm no unresolved gate, active task, branch, worktree, or release acceptance is silently excluded.
- [ ] Deliver a human Outcome with correctness, measured performance, deviations, version/Release links, installation/upgrade proof, and exact resource state.

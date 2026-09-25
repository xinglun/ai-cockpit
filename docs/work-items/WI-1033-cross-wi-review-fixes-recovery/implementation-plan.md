# WI-1033 Cross-WI Review Corrections Recovery Plan

## Goal

Complete Runtime-bound acceptance and lifecycle closure for the already implemented WI-1032 review fixes, preserve the original recovery history, and stop before release. Implementation is serial in this linked worktree; the independent PR review remains separate.

## Constraints

- Keep exactly one active Work Item, branch, worktree, and repository context.
- Do not rewrite WI-1031 or WI-1032 records. WI-1032 is archived as replaced/not_verified; WI-1031 verification is historically passed but its human close is still pending.
- Do not change Sentinel. Use the candidate release binary only for read-only compatibility inspection and prove protected state is byte-identical.
- Keep installed Runtime 0.2.113 fixed. Do not infer that an older Runtime understands or enforces candidate collaboration fields.
- Use Runtime CLI + Cargo/CI canonical gates; this Rust repository has no supported Make gate.
- Keep query routes read-only and durable changes behind explicit Runtime write commands.
- Do not release, publish, create or move tags, or upgrade the Runtime.
- Task 8 (Runtime snapshot binding across commits) is a separate serial Work Item after this one is integrated and cleaned up.

## Task 8 handoff: observed governance friction

These findings are evidence-backed follow-up inputs, not changes to WI-1033's implementation scope. Preserve Runtime authority and exact evidence binding; reduce repeated work only when equivalence is proven.

| Observed friction | Evidence in this Work Item | Candidate Task 8 improvement and guardrail |
| --- | --- | --- |
| A source commit makes prior Runtime verification stale, even when the change is narrow; refreshing evidence reruns the full gate. | The latest full run before the current review fix, at `cb842380`, executed 34/34 nodes in 560,091 ms, spawned 34 processes, reused 0, and is now stale after the Contract amendment plus the Contract-path containment fix. | Bind reusable checks to declared content inputs and affected nodes, not a blanket commit identity; reuse only when repository, Runtime, Contract, check command, policy, and every observed input still match. Required/protected checks remain fresh-only. |
| A prior broad user authorization is repeatedly presented as a new preflight confirmation after snapshot changes. | Preflight requested `confirm_review` for changed snapshots despite the existing authorization to continue through merge and exact cleanup, stopping before release; an identity-bound Runtime receipt still had to be appended for each exact snapshot. | Separate persistent bounded authority from snapshot-specific evidence. Permit automatic append-only rebinding only while Work Item, Contract authority/scope, base, and stop boundary remain identical; require a new human decision when any of those change. Never treat the rebind as verification or review evidence. |
| `preflight` and `status` can present different next actions around a failed lifecycle projection. | Preflight exposed `rerun_affected_checks` while status still blocked `run_verification` on `lifecycle_gate_failed`; only the official retry receipt restored a coherent verification action. | Have preflight, status, and command admission consume one versioned admission result with the same blocker set and explain the exact recovery transition. Keep blocked Outcomes append-only. |
| Required-scenario mapping failure can be discovered only after a long canonical run. | The first 34-node verification passed, but `finish` failed because three already-passing required scenarios lacked Summary coverage mappings. | Validate mapping completeness before expensive process execution; show the exact missing scenario-to-test/evidence entries. Do not infer a mapping or mark a scenario verified automatically without evidence. |
| `amend` and `revalidate-amendment` have overlapping semantics that are easy to mis-sequence. | `work-item amend` already appended a `contract_amendment_revalidation` checkpoint; an immediate separate `revalidate-amendment` was then rejected because Contract bytes had not changed again. | Make CLI help and output say whether amendment includes revalidation, and expose the one valid next action. Preserve the append-only checkpoint and do not ask the operator to retry an unchanged command. |
| Parallel collaboration primitives are only partially exposed to Agents. | CLI/MCP operations and capability documentation exist, but `.ai/agent-interface.json` exposes only the generic `mcp` capability (`tools` is null), and the ordinary Work Item guide does not explain when or how to use parallel coordination. | Make supported parallel/coordination capabilities and tool schemas discoverable through the canonical Agent interface; add a concise guide for compatibility checks, linked-worktree isolation, lease operations, impact/pause handling, and the serial fallback. Runtime remains authoritative and ordinary single-WI work must stay serial. |

## Remaining work

### 1. Establish the recovery Contract before implementation admission

- Add this specification and plan in the WI-1033 worktree.
- Activate the Runtime-generated recovery scaffold with the same bounded intent and source scope.
- Declare all twenty-four acceptance criteria, the twenty required scenarios with both expected results and verification plans, required evidence classes, and the correct canonical docs command.
- Cover the retirement-parity invariant: every required locale row binds the retirement receipt and not_verified state; any missing locale fails closed without inventing successful verification evidence.
- Run Runtime preflight and checkpoint only when the current Runtime admits them. Preserve any rejection without retrying unchanged inputs.

### 2. Revalidate existing implementation, repairing only demonstrated gaps

- Trace each independent-review finding to the implementation and a regression test.
- Include Contract-bound required-check coverage labels, no-follow evidence and active-Contract reads at registration/publication/inspection, and execution-repository/CoordinationStore common-directory identity as explicit review findings with pre-spawn negative tests.
- Add failing gate regressions for a valid active recovery successor and a replaced/not_verified archive; distinguish intermediate lineage from terminal close, retirement evidence from successful verification evidence, require complete locale parity, reject contradictory verification-receipt claims, and cover each missing-row case.
- Run focused negative and positive tests for trust identity, complete required-check coverage, crash recovery/live lock safety, evidence containment, provider/outcome selection, actual reuse, MCP identity, and query/write persistence.
- If a gap is found, add the failing regression first, implement the narrow repair, commit it separately, then re-query Runtime because the repository snapshot changed.
- Do not restate classification-function tests as proof of process reuse; count real spawned processes.
- Add a failing provider-registration regression where an old output is removed at a newer generation; prove the persisted event retains the old output ID and invalidation reaches direct and multi-level consumers.
- Add failing coordination-store regressions for traversal/mismatched request identities, including a traversal-shaped embedded registration Work Item ID on both creation and transition; assert registration identity is rejected before Contract fact lookup and request bytes stay unchanged, alongside non-Requested creation-state coverage.
- Add a deterministic directory-handle swap regression for Contract containment: open the active directory, move the opened directory outside the repository, replace its old in-repository pathname, and prove that the exact handle used by the read—not a fresh pathname lookup—controls containment. Reject before registration/event persistence and preserve bytes.
- Add a deterministic swap-back race regression: move the already-open active directory outside, open the Contract through that handle, restore it before the next check, and prove ancestor mutation detection plus registration/event digest binding rejects the bytes. Route verification-evidence Contract reads through the same digest-bound reader before parsing or trusting them.
- Ensure the Windows-only directory-rename denial regression is executed by the existing Windows CI job; pin that command with the repository gate-manifest test rather than treating the test source as hosted proof.
- Implement only after observing each regression fail for the expected reason; rerun the focused `collaboration_admission` and `coordination_store` integration suites after each fix.

### 3. Run canonical and object-project compatibility evidence

Run, in order, the Contract-declared checks:

- cargo fmt --all -- --check
- cargo test --locked --workspace
- cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
- python3 tests/ci/repository_gate_manifest_test.py
- cargo build --locked --release -p cockpit-cli --bin ai-cockpit
- python3 tests/acceptance/cross_wi_coordination_processes.py --binary target/release/ai-cockpit
- bash tests/docs/documentation_acceptance.sh
- bash tests/ci/governance_integrity_gate_test.sh

Use the candidate binary for Sentinel inspection. Record exact before/after Git tree, Contract/evidence digests, lifecycle Runtime identity, and coordination-store bytes; stop if any protected state changes.

### 4. Runtime handoff, review, merge, and cleanup

- Re-query inspect, status, doctor, and the Work Item Outcome before each lifecycle boundary.
- Run Runtime-bound verification only against the final committed snapshot. Keep every attempt and rejection receipt.
- Deliver an explicit Outcome that separates implementation, verification, independent review, merge, local cleanup, historical lineage close, and release status.
- Push the authorized branch, create/update the PR, wait for hosted checks, obtain independent review, merge, and perform exact branch/worktree cleanup only when Runtime admits each action.
- Resolve the WI-1031 → WI-1032 → WI-1033 historical chain with the Runtime's selected-successor-lineage mechanism only after all required archived-node facts are available. Do not manufacture close evidence.
- Stop before release. Once this WI and exact cleanup are complete, create a distinct serial Work Item for Task 8.

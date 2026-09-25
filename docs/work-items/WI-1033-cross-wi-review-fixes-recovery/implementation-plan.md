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
| A source commit makes prior Runtime verification stale, even when the change is narrow; refreshing evidence reruns the full gate. | The latest pre-amendment full run, at `faa3eb4`, executed 35/35 nodes in 512,387 ms, spawned 35 processes, reused 0, and became stale after the required-check environment Contract amendment and guard. | Bind reusable checks to declared content inputs and affected nodes, not a blanket commit identity; reuse only when repository, Runtime, Contract, check command, policy, and every observed input still match. Required/protected checks remain fresh-only. |
| A prior broad user authorization is repeatedly presented as a new preflight confirmation after snapshot changes, and a stale decision binding can reach hosted CI. | The previous receipt bound `preflightDecisionDigest=sha256:1c6980110d6452fa8ff819865ad28ba9c0d175200442ec13f0e38346fde8e63f`, while the final preflight digest became `sha256:87984c0a1a26d0916db4f2c1b13f031eb89189e9b24d41497391ad35de9cf55e`; CI run `36177143896` then blocked before repository gates with `preflight_decision_evidence_invalid` and `human_review`, even though scope and authority were unchanged. | Separate persistent bounded authority from snapshot-specific evidence. After the final preflight, append the decision binding for that exact decision and snapshot before push; show stale bindings and their precise repair action. Require a new human decision only when Work Item, Contract authority/scope, base, or stop boundary changes. Never treat a rebind as verification or review evidence. |
| `preflight` and `status` can present different next actions around a failed lifecycle projection. | Preflight exposed `rerun_affected_checks` while status still blocked `run_verification` on `lifecycle_gate_failed`; only the official retry receipt restored a coherent verification action. | Have preflight, status, and command admission consume one versioned admission result with the same blocker set and explain the exact recovery transition. Keep blocked Outcomes append-only. |
| Required-scenario mapping failure can be discovered only after a long canonical run. | The first 34-node verification passed, but `finish` failed because three already-passing required scenarios lacked Summary coverage mappings. | Validate mapping completeness before expensive process execution; show the exact missing scenario-to-test/evidence entries. Do not infer a mapping or mark a scenario verified automatically without evidence. |
| `amend` and `revalidate-amendment` have overlapping semantics that are easy to mis-sequence. | `work-item amend` already appended a `contract_amendment_revalidation` checkpoint; an immediate separate `revalidate-amendment` was then rejected because Contract bytes had not changed again. | Make CLI help and output say whether amendment includes revalidation, and expose the one valid next action. Preserve the append-only checkpoint and do not ask the operator to retry an unchanged command. |
| Parallel collaboration primitives are only partially exposed to Agents. | CLI/MCP operations and capability documentation exist, but `.ai/agent-interface.json` exposes only the generic `mcp` capability (`tools` is null), and the ordinary Work Item guide does not explain when or how to use parallel coordination. | Make supported parallel/coordination capabilities and tool schemas discoverable through the canonical Agent interface; add a concise guide for compatibility checks, linked-worktree isolation, lease operations, impact/pause handling, and the serial fallback. Runtime remains authoritative and ordinary single-WI work must stay serial. |
| An interrupted Runtime verification may have no durable attempt record. | The canonical 35-node run started after commit `0b7ae3b5` was intentionally sent SIGINT on 2026-09-25 after independent review found a P1; the host command returned exit 130, but the latest persisted Runtime attempt still dates to 14:18 and binds the prior head. | Persist an attempt envelope before the first process starts, append node results as they finish, and finalize interrupted runs with a distinct terminal state bound to the exact snapshot and signal. Never project an incomplete attempt as passed. |
| A required shared implementation file can be missing from the Contract scope even though it supports an already-approved acceptance behavior. | Preflight reported `scope_exceeded` for `crates/cockpit-verification/src/lib.rs`; the file supplies environment clearing and environment-bound command identity for the scoped composition runner. Runtime appended this path and its source in the Contract amendment from `sha256:29ab3c0bdcc87ec77a73c8443603333a55f21624e63a0d29051aa43fde4c1750` to `sha256:13c3c1a56540648a08fe9b9467ff5734ca5596f7c4488d436aa7dac27d7cbd3d`. | Detect missing support-file scope while preparing the implementation plan; distinguish an additive path amendment within the same approved intent from genuinely new scope, and never continue while Runtime reports `scope_exceeded`. |
| The hosted governance gate can surface missing lineage/parity inputs that are not obvious from the active worktree. | CI runs `36155139121` and `36174008746` both reported WI-1031 missing a terminal close decision, stale parity status in all three locales, and missing Japanese WI-1032/WI-1033 locations. In the latter run, the selected recovery, retirement, archive, and active-WI records were present only as local untracked Runtime outputs, not in the pushed candidate. | Before expensive hosted gates, compare the exact staged candidate against a clean checkout and enumerate every required lineage receipt and locale parity row. Preserve missing close evidence as a blocker; never synthesize it. |
| Platform-specific dependencies, types, and path assertions can remain hidden until hosted CI compiles and runs tests on a different target. | Run `36155139121` found `tempfile` missing from Windows library tests; run `36174008746` found a Windows-only `u32`/`i32` comparison and unconditional Unix `CString` import; run `36177143896` compiled the library but failed two tests that compared Windows `PathBuf` output to POSIX strings (`cargo_verification_defaults_to_home_target_when_not_declared` and `receipt_next_action_is_executable_and_retained_action_is_non_destructive`). The directory-handle containment regression itself passed in that run. | Add a target-matrix dependency, compile, and test preflight; compare paths by path semantics rather than host-specific string spelling, while retaining Windows CI as authoritative execution evidence. |
| The installed Runtime and source-built candidate Runtime can have different executable identities despite the same version string, so existing receipts are accepted locally but contradictory to the candidate. | On the same `0.2.113` Work Item snapshot, the installed Runtime digest was `sha256:c85632062eb5ef8f8b39f6154b765c9e2543fe6b7ca3f54e107a59c174843686`, while the PR/CI candidate was `sha256:a4eba1c4f69adf0c49fef593531c49863ef9b4e1139c782d812c312f893a86d0`. The installed CLI projected verification green; the candidate projected `evidence_contradictory`, and CI `36177143896` could not evaluate the quality gate until the stale preflight decision was resolved. | Bind preflight, recovery, verification, status, and hosted gates to one explicit executable digest; when changing Runtime identity, explain the required append-only retry and fresh verification. Do not equate matching version strings with binary compatibility or silently accept old-runtime evidence. |
| One shared lint failure is repeated across dependent package nodes, and the recorded stderr is hex encoded, adding diagnosis time. | Failed Runtime attempt `6fa0f43e8708185ae6c0b71e51725c92683483be18dbb99331bdab6d69552c7d` showed the same `composition.rs:810` Clippy error in four crate nodes; decoding the bytes exposed one root cause, and fixing it made all four crate lints pass. | Provide a readable, deduplicated root-cause summary while retaining every per-node result and raw output. Do not collapse distinct failures or discard original evidence. |
| A serial-only Work Item can be projected as incompatible when optional parallel compatibility is undeclared. | `work-item inspect` for WI-1033 returned `compatible: false`, reason `parallel_compatibility_not_declared`; the declared execution path is serial and does not request parallel coordination. | Separate serial readiness from optional parallel compatibility; absent parallel declarations must not block a valid single-WI serial flow. Keep parallel admission fail-closed when it is explicitly requested. |
| A lexical governance heuristic can flag test weakening based on cleanup wording rather than a weakened assertion. | During WI-1033 preflight, a test cleanup error string containing “remove” contributed to a `test_weakening` finding; changing only that diagnostic wording removed the finding without altering test assertions. | Report the exact matched evidence and distinguish test behavior/assertion changes from diagnostic text; retain fail-closed behavior for actual weakening. |

## Remaining work

### 1. Establish the recovery Contract before implementation admission

- Add this specification and plan in the WI-1033 worktree.
- Activate the Runtime-generated recovery scaffold with the same bounded intent and source scope.
- Declare all twenty-nine acceptance criteria and twenty-four required scenarios with both expected results and verification plans, required evidence classes, and the correct canonical docs command.
- Cover the retirement-parity invariant: every required locale row binds the retirement receipt and not_verified state; any missing locale fails closed without inventing successful verification evidence.
- Run Runtime preflight and checkpoint only when the current Runtime admits them. Preserve any rejection without retrying unchanged inputs.

### 2. Revalidate existing implementation, repairing only demonstrated gaps

- Trace each independent-review finding to the implementation and a regression test.
- Include Contract-bound required-check coverage labels, no-follow evidence and active-Contract reads at registration/publication/inspection, and execution-repository/CoordinationStore common-directory identity as explicit review findings with pre-spawn negative tests.
- Add failing gate regressions for a valid active recovery successor and a replaced/not_verified archive; distinguish intermediate lineage from terminal close, retirement evidence from successful verification evidence, require complete locale parity, reject contradictory verification-receipt claims, and cover each missing-row case.
- Run focused negative and positive tests for trust identity, complete required-check coverage, crash recovery/live lock safety, evidence containment, provider/outcome selection, actual reuse, MCP identity, and query/write persistence.
- If a gap is found, add the failing regression first, implement the narrow repair, commit it separately, then re-query Runtime because the repository snapshot changed.
- Do not restate classification-function tests as proof of process reuse; count real spawned processes.
- Add a failing provider-registration regression where an old output is removed at a newer generation; prove the persisted event retains the old output ID and invalidation reaches direct and multi-level consumers. Also cover a schema-v1 legacy Impact event with no `outcomeIds`: after the provider removes an output, current consumer declarations must retain transitive invalidation while unrelated work remains admissible.
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

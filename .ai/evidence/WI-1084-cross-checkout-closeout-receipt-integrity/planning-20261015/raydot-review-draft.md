# WI-1084 — #1004 recovery-receipt integrity draft for Raydot review

Status: design and Contract draft only. No implementation code or regression test has been written or run. No claim of a reproduced RED, fix, formal verification or Issue acceptance is made.

## Authorization and isolation

Ray authorized an independent WI/branch/worktree for the bounded #1004 fix; Raydot directed the implementer to prepare this Contract for ordinary technical review first. Base source is `bcd76ae76f61650a890267f511099c96dc6a47fd`. Branch is `codex/wi1084-closeout-receipt-integrity`; worktree is `/workspace/ai-cockpit-wi1084-closeout-receipt-integrity`. Build output is reserved at `target/wi1084-closeout-receipt-integrity/` in that worktree: fresh isolated target, one build job, no incremental compilation and debug info disabled, with the existing 900-second verification limit. No build has started. Merge, deletion, human close, Task9 and Sentinel changes are outside authority.

The WI number was selected after normal fetch, scanning all six preexisting worktrees, locally known refs and GitHub heads/open PRs. The maximum observed WI was 1083; the normal Runtime scaffold reservation for the complete WI-1084 ID succeeded. No global numbering allocator is claimed.

## Original reproduction and what is actually available

Issue #1004 comment 6186749658 reports an independently built macOS arm64 CLI at exact source bcd76ae7, binary SHA-256 `86188301fc97d44d67765ca340e7a8b6d4b8a10d9830245613ed9064e4638900`. It reports 11/11 payload hashes, successful plan/recovery/idempotent retry, and a historical not_ready projection. Removing only the destination recovery receipt (raw SHA-256 `c5096578f6f8d5bad9def78929ad34cea05abe9a92342e2864f60d13b2d4dba3`) left status exit 0, lifecyclePhase=closed and blocking=false.

These are the reporter's original public observations, not this environment's rerun. The full Mac acceptance report/manifest and Sentinel files under /Users are not available here. They were not reconstructed or accessed. The public Issue remains OPEN; no time TTL was specified or added. PR #1006 already disclosed the sole-receipt deletion ambiguity.

## Static data-flow evidence

At exact bcd76ae7:

1. `historical_compatibility.rs:23–85` validates a close record and calls `recovered_closeout_files_valid` before accepting canonical or legacy human-close vocabulary.
2. `historical_compatibility.rs:92–104` looks only for the canonical cross-checkout recovery receipt and returns true on NotFound to preserve ordinary close compatibility. With that file missing, the entire imported-file/hash/manifest check is skipped.
3. `lifecycle.rs:6406–6470` persists one recovery receipt. `lifecycle.rs:6513–6696` imports every non-close payload, persists that receipt, validates, then installs the exact source close bytes as the commit marker. It adds no separately required persistent recovery-origin binding to the copied close.
4. `status_projection.rs:1207–1237, 1375–1384` uses the boolean close validity to select closed vs archived and add/remove archived_work_item_pending_close. This explains a source path consistent with the report; an automated RED must still prove the complete path.
5. Existing unit cases `recovered_close_is_not_projected_closed_after_payload_disappears` and `tampered_recovery_receipt_cannot_omit_missing_archived_payload` retain the recovery receipt while damaging its payload. They do not cover removing the sole receipt.

## Proposed bounded change (not yet implemented)

Keep all imported historical close/archive/Contract/verification/payload bytes exact. Introduce a repository-local immutable recovery-origin binding for newly recorded recoveries, alongside an explicitly versioned recovery receipt. The origin binding must survive deletion of the required receipt, identify its canonical path and bind the repository, WI, canonical Contract, source snapshot, provider facts and original close/payload identity. A versioned receipt references the origin binding by digest; validate both required records and their exact critical bindings before treating the copied close as terminal. Use an acyclic binding: do not create mutually self-referential file digests. Reuse the existing immutable installer and serialized lifecycle lock; install and recheck the required provenance/receipt before publishing the close marker. Published paths remain preserved on failures.

Normal supported retry must either restore the same bound records from validated source facts or report a precise conflict; it must never overwrite a different published record or backdate timestamps. Review should settle the smallest stable field/digest shape before code. No new flags, provider client, generic evidence registry, system configuration, timestamp TTL or unrelated status rewrite is proposed. Implement in the existing recovery writer and shared close validator; CLI/MCP keep consuming the same service.

## Compatibility boundary for technical review

- Ordinary current and ordinary legacy historical close records must continue to work without a fabricated recovery dependency.
- Existing v1 recovery receipt and historical source bytes are not silently rewritten or migrated. If a legacy receipt disappeared before any independent binding existed, its prior recovery origin is not inferable from the copied ordinary close alone. State that limitation explicitly. New recovery through the candidate writer must have the additional required binding.
- Removing both all provenance and all dependent proof cannot be made detectable from an otherwise indistinguishable legacy close without another trusted anchor. This task targets loss of the unique required recovery receipt after a newly bound recovery; do not claim a general tamper-proof store.
- A recovered checkout used as a later recovery source must first pass the shared current validator; add a regression for that flow rather than copying a destination-specific receipt into a conflicting canonical path.
- Historical not_ready and historical_evidence_not_revalidated are preserved; green CI does not replace historical or adopter acceptance.

## RED-first regression plan

All scenarios below are planned, not executed. Assert state, blockers and admission; status exit 0 is allowed.

| Scenario | RED/acceptance assertion |
| --- | --- |
| Exact sole-receipt deletion | Native recovery in independent Git common dirs; confirm closed/non-blocking; remove only receipt; query succeeds but must be archived/blocking with an integrity diagnostic. Assert remaining payload bytes are unchanged. First show this fails on unmodified bcd76ae7. |
| Overlapping work admission | After the deletion, an overlapping new WI must not be falsely admitted because the historical scope appeared closed. Distinguish the intended close-integrity rejection from dirty-tree or topology errors. |
| Required binding/receipt integrity | Missing, malformed, non-regular or wrong WI/repository/Contract/source/close/provider binding fails closed. Required-record checks must not fall back to ordinary-close compatibility. |
| Positive and retry | Valid recovery, repeated recovery changedPaths=[], restored supported proof, and a recovered checkout as a later source preserve exact old bytes and correct closed state. |
| Historical + ordinary compatibility | Historical producer Runtime differs: preserve not_ready/unknown labels; ordinary current and legacy closes remain accepted without provenance. Legacy v1 limitation is explicit. |
| Failure preservation | Retain existing interruption, precommit and postcommit failure, racing target and source immutability cases. No destructive rollback, no replacement of conflicting canonical files. |
| Transport parity | Add narrow CLI and MCP cases for receipt loss; both share repository state validation and do not assert nonzero status exit as the defect. |

After Raydot review and fresh Runtime action admission, execute the narrow failing case first, then the minimal writer/validator change, focused recovery regressions and the existing Contract workspace verification plus required repository gates. Retain exact stdout/stderr, commands, source/Runtime/toolchain binding, counts, exit codes and timeout evidence; no weakened assertions or protected-gate changes.

## Actual parallel capability observation

The candidate Runtime implements the CLI coordination, declaration and slot surfaces. The initial scope-only compatibility read was true; that was not execution admission. With the complete four-class concurrency boundary, the authoritative fresh `work-item inspect` reports compatible=false: WI-1070 through WI-1074 have unknown boundaries and absent parallel declarations. No registration, lease, extra execution worker or composition was started. Do not change those WIs or hide them in another checkout. The proposed fallback is one serial implementation worker for WI-1084 while WI-1078 remains Retained and under separate review; any true cross-WI parallel execution needs explicit compatible declarations and fresh admission first. Integration remains serialized.

## Review and current Runtime boundary

Current Contract digest: `sha256:507bedef85c7d3cb7ff2b8fd3a885d5441384ea23b8d6e4b010fa9be8c150107`.
Runtime source bcd76ae7, binary SHA-256 `0c9b079a493c47552338f0bb1d5bcc0138d4025d02ecd1f495882f28e63fc1db`.

Prepared start was admitted and recorded the initial checkpoint. Subsequent supported amendments completed the draft; current action admission is needs_human_decision, with policy review for the changed scope/boundary/scenarios. Safe actions are record_governance_controls, record_usage and refresh_status. Scenario results are all unverified; intent alignment and required documentation projections are not yet supplied. These are normal pending draft facts, not a passed implementation.

Raydot technical review is pending. It is distinct from identity-bound human confirmation or a human close decision. No WI-1078/1081 approval is reused. Before implementation, resolve the current Native review requirements against this exact Contract and the then-current material, and re-query admission.

Routine failed argument/schema/digest probes are preserved alongside their corrected accepted requests. No rejected attempt is claimed as an accepted Contract update.

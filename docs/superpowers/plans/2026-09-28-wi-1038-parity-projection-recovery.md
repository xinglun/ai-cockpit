# WI-1038 Parity Projection Recovery Plan

> Implementation is serial in the active WI-1038 worktree. Preserve immutable WI-1037 records and stop before release.

**Goal:** Resolve PR #997's stale parity registration without hiding its historical ordering, then complete Task 8 merge and cleanup under Runtime admission.

**Contract:** `.ai/work-items/active/WI-1038-task8-parity-projection-recovery.contract.json`

## Bounded steps

1. Add a real-Git positive case for a valid WI-1037 → WI-1038 successor recovery whose Contract scope and Summary changed paths both include all three parity ledgers. Expect only an explicit historical warning for the original late registration.
2. Add a negative case proving that an unowned parity document remains a blocking stale-registration error. Keep malformed, foreign, mismatched, and partial identity fail-closed through the existing recovery checks.
3. Update the gate with the smallest successor ownership predicate; do not weaken the generic prearchive registration rule.
4. Correct WI-1037's three-language archived-but-awaiting-merge projection, add WI-1038's three-language lifecycle projection, and keep the lifecycle status cell stable across archive transition.
5. The declared focused governance, documentation, and parity checks passed serially on the candidate snapshot. Runtime-bound verification passed after switching from the released binary to the current source-matched Runtime digest; no package-suite rerun was performed.
6. Runtime admitted and archived WI-1038. The real post-archive parity check then exposed Git introduction-order mismatch between the parity row and its verification evidence. Preserve all historical evidence, commit the projection before the evidence path, and rerun only the affected post-archive gates. Then inspect exact-head required CI for PR #997; merge and exact Task 8 cleanup only when admitted, and require Runtime `readyOnBase` before starting Task 9.

## Review boundaries

- No change to WI-1037 archive, Summary, Outcome, events, or verification evidence.
- The historical warning must remain visible; no blanket late-registration waiver.
- No release build, version change, tag, or publication. Hand off for human release review after Task 8 cleanup and Task 9 readiness boundary.

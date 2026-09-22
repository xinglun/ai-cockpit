# Verification failure recovery

## Applicability

Load this guide only when a declared verification fails, times out, becomes
stale, or its evidence is invalidated. It covers diagnosis and governed
recovery, not ordinary implementation or release acceptance.

## Authoritative inputs

Use the exact failed verification command and output, Runtime `status`,
`inspect`, and `doctor` projection, the affected Contract and snapshot
digests, and the evidence freshness/invalidity reason. Preserve the Runtime
identity that produced the receipt. Do not infer a cause from a guide or
silently retry with a different executor.

## Operations

1. Preserve the failure receipt, logs, exit status, timestamps, and repository
   identity before diagnosing.
2. Ask the Runtime for the current blocker, issue category, safe action, and
   human decision requirement.
3. Classify the result as missing, unknown, malformed, unsupported, stale, or
   contradictory according to the Runtime projection; keep compatibility
   problems distinct from fabricated failure.
4. Apply only an admitted recovery action. Re-run the smallest declared check
   that can distinguish the suspected cause, then re-query before any next
   boundary.
5. Deliver a blocked or recovered Outcome; never report recovery as complete
   merely because a retry started.

## Success conditions

The failure and recovery evidence remain bound to the same authorized
identity, the Runtime accepts the resulting evidence as current, and a fresh
projection admits continuation. The Outcome states what failed, what was
recovered, what remains unknown, and the next action.

## Failure evidence

Retain the original failure and every retry receipt, including timeout or
process-start evidence. Keep incompatible historical formats as unsupported
compatibility issues, not as contradictions. A retry that is rejected must
retain the changed admission digest and rejection reason.

## Continue or stop

Continue only when the Runtime provides a current admitted recovery action and
the Contract still covers it. Stop for changed scope, missing authority,
contradictory evidence, a required human decision, or a recovery that would
overwrite the original evidence. Return to the ordinary guide only after the
Runtime shows a fresh, successful verification state.

## Reference

See [affected verification](../../docs/reference/affected-verification.md),
[operation-time policy reevaluation](../../docs/reference/operation-time-policy-reevaluation.md),
and the [agent workflow](../../docs/reference/agent-workflow.md).

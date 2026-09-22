# Ordinary Work Item

## Applicability

Use this guide for ordinary repository implementation, declared verification,
archive, and local cleanup when the current Runtime projection identifies
`ordinary-work-item`. It is the default route when the current Runtime
projection selects the ordinary path.

Do not use it to interpret failed, timed-out, stale, or invalid evidence, or
when Runtime selects a specialized recovery, external-resource, or
artifact-acceptance route. Load the selected conditional guide instead.

## Authoritative inputs

Read the active Contract and Runtime `inspect`, `status`, and `doctor` output
for the explicit repository path. Use the current `safeActions`, blockers,
evidence freshness, action explanation, Contract digest, and repository
snapshot. Use declared verification commands and the relevant Reference page.
Do not infer permission from this guide or hand-edit generated records.

## Operations

1. Confirm the Runtime identity and repository binding, then inspect the
   current status before touching files.
2. Make only the Contract-scoped change. Keep tests and evidence intact.
3. Re-query the Runtime before each lifecycle boundary and follow the action
   it currently admits; a recommendation is only guidance.
4. Run the declared checks at the scope required by the change. Preserve
   command, exit status, logs, and identity for every failure.
5. At handoff, report implementation, projection, and host delivery as
   separate Outcome facts.

## Success conditions

The current Contract and snapshot are bound, required checks have current
evidence, no blocker or unresolved human decision remains, and the Runtime
admits the next operation. The visible Outcome contains the complete status,
evidence, unknowns or risks, decision, verification, impact, and next action.

## Failure evidence

Keep the exact failing command, exit status, output path, Runtime projection,
Contract/snapshot digests, and any invalidated receipt. Do not replace a
yellow or red state with a prose claim of completion. If the failure is a
verification or evidence failure, switch to
[`verification-failure-recovery`](verification-failure-recovery.md).

## Continue or stop

Continue only after a fresh Runtime query admits the operation and all
required inputs are present. Stop for missing authority, unknown or
contradictory evidence, stale admission, an out-of-scope defect, or a human
decision. Ask for a real decision when the Runtime says one is required; do
not turn a recommendation into approval.

## Reference

See [agent workflow](../../docs/reference/agent-workflow.md), [how to read
status](../../docs/reference/how-to-read-cockpit-status.md), and the
[command reference](../../docs/reference/commands.md).

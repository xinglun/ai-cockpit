# Ordinary Work Item

Use this route when Runtime selects ordinary implementation, verification,
archive, or local cleanup. For failed/stale evidence, follow the selected
recovery guide.

## Governed work

Read the active Contract and query `inspect`, `status`, and `doctor` with an
explicit `--repo`. Runtime `safeActions`, blockers, evidence freshness, and
action explanation govern each step; this guide is not permission. Change only
Contract scope, preserve evidence/history, and re-query at lifecycle boundaries.
Run declared checks. Keep failures and invalidated receipts; never hand-edit
generated records. At handoff, give the separate human Outcome required by
`AGENTS.md`. Queries are read-only; `preflight` is an explicit, idempotent
write and does not run verification.

## Before verification

Before launching a declared check, inspect Runtime `work-item status` and
`work-item validate` and inventory its formal receipt. Reuse only fresh, complete
evidence; otherwise follow Runtime's admitted next action. See the
[verification evidence reuse procedure](../../docs/reference/agent-workflow.md#verification-evidence-reuse)
for identity bindings, targeted reruns, and hosted-evidence boundaries.

When fresh status recommends `record_governance_controls`, record only the
explicitly supplied evidence with `work-item controls --repo <repository>
--id <work-item> --input <json>` (or MCP `work_item_controls`). Both write
surfaces re-check current Runtime action admission. Re-query status afterward;
required controls that remain incomplete do not admit `finish`.

## Serial and cross-Work-Item use

Keep lifecycle and snapshot-changing writes serial. Independent checks may run
in parallel on a fixed input snapshot only when dependencies are ready, outputs
are isolated, and resource limits allow it; otherwise serialize. `verify
--workers N` caps Runtime workers; use `1` for serial verification. Reuse a fresh
producer receipt instead of rerunning its checks. A negative parallel
compatibility result denies fan-out only; continue admitted serial work.
Cross-Work-Item coordination prerequisites and discovery are in the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Coordination inspection is read-only. Register, report impact, publish an
outcome, request/acknowledge/safely pause, resume, recover, and acquire or
relinquish leases only through their explicit Runtime write actions. Bind
records to the current repository, Contract, Runtime, and generation; reject
stale requests.
Refresh dependency admission immediately before affected actions; unrelated
actions may continue only if their own refreshed admission allows. Recovery
appends a resolution; retain the original event.

The repository-bound installed Runtime owns lifecycle decisions. Candidate
collaboration writes require tools actually implemented by that candidate;
readable new fields do not prove compatibility. See the
[agent workflow](../../docs/reference/agent-workflow.md) for the full discovery
and safe-pause sequence.

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

One Work Item runs serially by default. Keep lifecycle and snapshot-changing
writes serial. Independent checks may run in parallel on a fixed input snapshot
only when dependencies are ready, outputs are isolated, and resource limits
allow it; otherwise serialize. `verify --workers N` bounds workers (`1` is
serial). Before cross-Work-Item fan-out, discover current CLI help plus MCP
`tools/list` schemas. Confirm candidate Runtime support and acquire a Runtime
slot lease. If unsupported, do not emulate constraints: use admitted serial
work or stop. A negative compatibility result denies fan-out only. Reuse a
fresh producer receipt instead of rerunning its checks.
See the [agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination)
for declarations and safe operations.

Coordination inspection is read-only; registration, impact/outcome, pause,
resume/recovery, and lease mutations use explicit Runtime actions bound to the
current repository, Contract, Runtime, and generation. Refresh dependency
admission before affected actions; unrelated work continues only on its own
refreshed admission. Recovery appends a resolution and retains the event. The
installed Runtime owns lifecycle decisions; readable fields do not prove
candidate support, and unsupported constraints must not be ignored or emulated.

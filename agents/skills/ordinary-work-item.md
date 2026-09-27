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

One Work Item keeps a serial path; lifecycle and snapshot-changing writes stay
serial. Work Item-bound `verify` defaults to `--workers 1`; explicit
`--workers >1` fails closed until Runtime verifies per-node dependency
readiness and output isolation. Do not split required gates across concurrent
commands on one checkout.

Parallelize independent checks only on an immutable snapshot, with ready
dependencies, isolated outputs, and bounded resources. CI jobs may fan out as
siblings only with the same route/source identity and no dependency edge. Keep
receipt producer-consumer serial; reuse fresh matching receipts.

Cross-Work-Item fan-out needs supported CLI/MCP, compatible declarations, and
a Runtime slot lease. If unsupported/unknown, use admitted serial work or stop;
a negative compatibility result denies fan-out only. See the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Coordination inspection is read-only; registration, impact/outcome, pause,
resume/recovery, and lease mutations use explicit Runtime actions bound to the
current repository, Contract, Runtime, and generation. Refresh dependency
admission before affected actions; unrelated work continues only on its own
refreshed admission. Recovery appends a resolution and retains the event. The
installed Runtime owns lifecycle decisions; readable fields do not prove
candidate support, and unsupported constraints must not be ignored or emulated.

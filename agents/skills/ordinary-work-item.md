# Ordinary Work Item

Use this route when Runtime selects ordinary implementation, verification,
archive, or local cleanup. For failed/stale evidence, follow the selected
recovery guide.

## Governed work

Read the active Contract; query `inspect`, `status`, and `doctor` with `--repo`.
Runtime `safeActions`, blockers, freshness, and explanation govern actions;
this guide is not permission. Stay within Contract scope, preserve
evidence/history, and re-query at lifecycle boundaries. Run declared checks;
keep failures and never hand-edit generated records. Queries are read-only;
`preflight` is explicit, idempotent, and does not verify. At handoff, deliver
the separate human Outcome required by `AGENTS.md`.

## Before verification

Before launching a declared check, inspect Runtime `work-item status` and
`work-item validate` and inventory its formal receipt. Reuse only fresh, complete
evidence; otherwise follow Runtime's admitted next action. See the
[verification evidence reuse procedure](../../docs/reference/agent-workflow.md#verification-evidence-reuse)
for identity bindings, targeted reruns, and hosted-evidence boundaries.

When status admits `record_governance_controls`, submit only explicit evidence
via `work-item controls --repo <repository> --id <work-item> --input <json>` or
MCP `work_item_controls`; both re-check admission. Refresh status; incomplete
required controls block `finish`.

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

Before cross-Work-Item use, inspect the local capability manifest, current CLI
help, and MCP `tools/list` schemas; use `ai-cockpit capability show --repo
<repository>`. Readable fields or older Runtime versions do not prove support.

Cross-Work-Item fan-out needs supported CLI/MCP, compatible declarations, and
a Runtime slot lease. If unsupported/unknown, use admitted serial work or stop;
a negative compatibility result denies fan-out only. See the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Inspection is read-only. Registration, impact/outcome, pause/resume/recovery,
and lease mutations are explicit writes bound to repository, Contract, Runtime,
and generation. Refresh dependency admission before affected actions; unrelated
work needs its own fresh admission. Recovery appends a resolution and retains
the event. Runtime owns lifecycle decisions; never infer support from fields or
ignore/emulate unsupported constraints.

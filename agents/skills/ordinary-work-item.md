# Ordinary Work Item

Use when Runtime selects ordinary implementation, verification, archive, or
local cleanup. For failed/stale evidence, use the selected recovery guide.

## Governed work

Read the active Contract; query `inspect`, `status`, and `doctor` with `--repo`.
Runtime decides admission; this guide grants none. Stay in scope, preserve
history and failures, re-query at lifecycle boundaries, and never hand-edit
generated records. Queries are read-only; `preflight` is idempotent, not
verification. At handoff, deliver the human Outcome required by `AGENTS.md`.

## Before verification

Before launching a declared check, inspect Runtime `work-item status` and
`work-item validate`, inventory its formal receipt, and reuse only fresh,
complete evidence; otherwise follow Runtime's admitted next action. See
[verification evidence reuse](../../docs/reference/agent-workflow.md#verification-evidence-reuse)
for identity bindings, targeted reruns, and hosted boundaries.

When status admits `record_governance_controls`, submit explicit evidence via
`work-item controls --repo <repository> --id <work-item> --input <json>` or MCP
`work_item_controls`; both re-check admission. Refresh status; incomplete
required controls block `finish`.

## Serial and cross-Work-Item use

One Work Item is serial by default; lifecycle/snapshot writes and receipt
production/consumption stay serial. Verify defaults to `--workers 1`; parallel
workers fail closed until Runtime verifies dependencies and output isolation.
Independent CI jobs may fan out with ready dependencies, isolated outputs, and
bounded resources.

Cross-Work-Item work needs supported CLI/MCP, compatible declarations, linked
worktrees, registration, a slot lease, and fresh per-action admission. Check
`capability show`, CLI help, and MCP `tools/list`; otherwise use admitted
serial work or stop. See the [agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Inspection is read-only; registration, reports, coordination, leases, and drift
recovery are explicit writes. Recovery appends events; re-admit unrelated work.
For closeout transfer, see the [recovery commands](../../docs/reference/commands.md#cross-checkout-work-item-closeout-recovery).

## Plan changes and environment drift

Amend plans with `work-item amend --request`, a reason, and current
`expectedContractDigest`; `work-item amendments` reads append-only history.
Identity, lifecycle, observations, and evidence stay protected; sensitive edits
rerun policy and invalidate affected checks.

Before dependent actions, check environment drift read-only; record changes
before refreshing admission. Shared events are durable, append-only, and
generation-bound; the request-scoped ledger is not a cross-process bus. Admit
unrelated work independently; use serial execution if capability is absent.
See the [agent workflow](../../docs/reference/agent-workflow.md).

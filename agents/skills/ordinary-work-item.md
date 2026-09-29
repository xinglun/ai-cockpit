# Ordinary Work Item

Use this route when Runtime selects ordinary implementation, verification,
archive, or local cleanup. For failed/stale evidence, follow the selected
recovery guide.

## Governed work

Read the active Contract; query `inspect`, `status`, and `doctor` with `--repo`.
Runtime admits actions; this guide grants none. Stay in scope, preserve
evidence/history and failures, re-query at lifecycle boundaries, and never
hand-edit generated records. Queries are read-only; `preflight` is idempotent,
not verification. At handoff, deliver the separate human Outcome required by
`AGENTS.md`.

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

One Work Item is serial by default; lifecycle and snapshot-changing writes
stay serial. Work Item-bound verify defaults to --workers 1; explicit
--workers >1 fails closed until Runtime verifies per-node dependency readiness
and output isolation. Independent CI jobs may fan out as siblings with ready
dependencies, isolated outputs, and bounded resources. Keep receipt
producer-consumer serial; reuse fresh matching receipts.

Cross-Work-Item work needs supported CLI/MCP, compatible declarations, linked
worktrees, registration, a slot lease, and fresh per-action admission. Check
`capability show`, CLI help, and MCP `tools/list`; fields or an older Runtime do
not prove support. If unavailable, use admitted serial work or stop. See the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Inspection is read-only; registration, impact/outcome, pause/resume/recovery,
lease, and drift-event operations are explicit writes. Recovery appends events;
unrelated actions need fresh admission.

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

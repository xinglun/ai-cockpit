# Ordinary Work Item

Use this route when Runtime selects ordinary implementation, verification,
archive, or local cleanup. For failed/stale evidence, follow the selected
recovery guide.

## Governed work

Read the active Contract and query `inspect`, `status`, and `doctor` with
`--repo`. Runtime admission governs actions; this guide is not permission.
Stay in scope, preserve evidence/history, re-query at lifecycle boundaries,
and keep failures. Never hand-edit generated records. Queries are read-only;
`preflight` is idempotent and does not verify. At handoff, deliver the separate
human Outcome required by `AGENTS.md`.

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

One Work Item is serial by default; `verify` uses one worker. More workers
require Runtime-proven dependency readiness and output isolation. Parallel
checks/CI require an immutable shared identity, independent work, and isolated
outputs; reuse fresh matching receipts.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations,
linked worktrees, registration, a slot lease, and fresh admission before each
affected action. Check `capability show`, CLI help, and MCP `tools/list`; fields
or an older Runtime do not prove support. If unavailable, use admitted serial
work or stop. See the [agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Inspection is read-only. Registration, impact/outcome, pause/resume/recovery,
lease, and drift-event changes are explicit writes. Recovery appends and keeps
the event; unrelated actions require their own fresh admission.

## Plan changes and environment drift

Use `work-item amend --request` with a reason and current
`expectedContractDigest`; `work-item amendments` reads append-only history.
Identity, lifecycle, observed facts, and evidence are protected; sensitive edits
re-run policy and invalidate affected checks.

Before dependent actions, check environment drift read-only and explicitly
record observed changes before refreshing admission. Shared events are durable,
append-only, and generation-bound; the request-scoped observation ledger is not
a cross-process bus. Admit unrelated work independently; use serial execution
when the capability is unavailable. See the [agent workflow](../../docs/reference/agent-workflow.md).

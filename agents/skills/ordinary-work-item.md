# Ordinary Work Item

Use for Runtime-selected implementation, verification, archive, or cleanup.
For failed/stale evidence, use recovery guide.

## Governed work

Read the active Contract; query `inspect`, `status`, and `doctor` with `--repo`.
Runtime admits; this guide grants none. Preserve scope, history,
and failures; re-query at lifecycle boundaries. Never hand-edit generated
records. Queries are read-only; `preflight` is not verification. At handoff,
deliver the human Outcome per `AGENTS.md`.

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

## Material review

See the [material review commands](../../docs/reference/commands.md). Stage 1 has no opt-in.

## Serial and cross-Work-Item use

Keep lifecycle, snapshot-changing, and receipt producer-consumer actions
serial. Verification defaults to `--workers 1`; parallelism requires
Runtime-verified dependency readiness and output isolation. Independent CI
jobs may fan out only with ready dependencies, isolated outputs, and bounded
resources.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations,
linked worktrees, registration, a slot lease, and fresh admission. Check
`capability show`, CLI help, and `tools/list`; declarations alone do not prove
support. Otherwise use admitted serial work or stop. See the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).

Inspection is read-only; registration, reports, coordination, leases, and
drift recovery are explicit writes. Recovery appends events; re-admit unrelated
work. For closeout transfer, see the [recovery commands](../../docs/reference/commands.md#cross-checkout-work-item-closeout-recovery).

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

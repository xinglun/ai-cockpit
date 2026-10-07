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

Use `work-item material-review plan --repo <repository> --id <work-item>` or
MCP `work_item_material_review_plan` for the read-only canonical request. It
requires committed, clean non-`.ai` source and preserves scanner Findings and
raw Unknowns; a plan is not a decision and does not discharge an Unknown.
`work-item material-review record --repo <repository> --id <work-item> --input <decision.json>` and MCP `work_item_material_review_record` share the same typed repository service. Recording is admitted only when the exact
Contract opt-in and current Runtime action admission both allow it. The
decision records `assurance=self_declared`; its `reviewerActor` is a claim,
not authenticated identity, human approval, provider/host verification, or
release approval. It never labels machine-Unknown material Clean. At Stage 1,
the opt-in is absent, so no material Unknown can be discharged. Keep the raw
scanner result and residual risk visible. See the English, Chinese, and
Japanese plan/record entries in the [command reference](../../docs/reference/commands.md).

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

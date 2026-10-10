# Ordinary Work Item

## Applicability

Use for Runtime-selected implementation, verification, archive, and local
cleanup. A failed, timed-out, stale, or invalid verification uses the
[verification recovery guide](verification-failure-recovery.md). For a Work
Item with a bound external Provider resource, use the
[provider finalization guide](provider-resource-finalization.md). Explicitly
authorized release or upgrade acceptance uses the
[release acceptance guide](release-upgrade-acceptance.md).

## Authoritative inputs

Read the active Contract and query `inspect`, `status`, `doctor`, and
`agent doctor --json` with an explicit `--repo <repository>`. Runtime output is
authoritative for action admission; this guide grants no action. Use the
current Runtime help and capability surface before forming arguments.

Before launching a declared check, inspect Runtime `work-item status` and
`work-item validate`, inventory its formal receipt, and reuse only fresh,
complete evidence. Otherwise follow Runtime's admitted next action. See the
[verification evidence reuse](../../docs/reference/agent-workflow.md#verification-evidence-reuse)
reference for identity bindings, targeted reruns, and hosted boundaries.

## Operations

When status admits `record_governance_controls`, submit explicit evidence via
`work-item controls --repo <repository> --id <work-item> --input <json>` or MCP
`work_item_controls`; both re-check admission. Refresh status; incomplete
required controls block `finish`.

For material review, use the [material review commands](../../docs/reference/commands.md).
Stage 1 has no opt-in.

One Work Item is serial by default; keep lifecycle, snapshot-changing, and
receipt producer-consumer actions serial. Verification defaults to
`--workers 1`; parallelism requires Runtime-verified dependency readiness and
output isolation. Independent CI jobs may fan out only with ready dependencies,
isolated outputs, and bounded resources.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations,
linked worktrees, registration, a slot lease, and fresh admission. Discover
support with `capability show`, CLI help, and MCP `tools/list`; declarations
alone do not prove support. Declared fields or an older Runtime do not prove
support. If unavailable, use admitted serial work or stop. See the
[agent workflow](../../docs/reference/agent-workflow.md#serial-fallback-and-cross-work-item-coordination).
Inspection is read-only; registration, reports, coordination, leases, and drift
recovery are explicit writes. Recovery appends events; re-admit unrelated work.

Amend plans with `work-item amend --request`, a reason, and the current
`expectedContractDigest`; `work-item amendments` reads append-only history.
Identity, lifecycle, observations, and evidence stay protected; sensitive edits
rerun policy and invalidate affected checks. Before dependent actions, check
environment drift read-only; record changes before refreshing admission.
Shared events are durable, append-only, and generation-bound; the
request-scoped ledger is not a cross-process bus. Use serial execution if
capability is absent.

For closeout transfer, use the
[recovery commands](../../docs/reference/commands.md#cross-checkout-work-item-closeout-recovery).

## Success conditions

Continue only when the Runtime admits the requested action and its required
inputs are present. Re-query before an action after the Contract or repository
snapshot changes. At handoff, deliver the human Outcome per `AGENTS.md`.

## Failure evidence

Preserve scope, history, prior failures, receipts, and their bindings. Never
hand-edit Runtime-generated records. Queries are read-only; `preflight` is not
verification. Report blockers and unknowns rather than converting them into a
success claim.

## Continue or stop

Preserve and report unknowns; unknowns alone do not stop an operation admitted
by the current Runtime. Stop for missing authority, contradictory evidence, or
a required human decision, and show the Runtime reason. Continue only when the
Runtime admits the requested action and its required inputs are present.

## Reference

See the repository [entry point](../../AGENTS.md), [Runtime overview](../../.ai/README.md),
[agent workflow](../../docs/reference/agent-workflow.md), and
[command reference](../../docs/reference/commands.md).

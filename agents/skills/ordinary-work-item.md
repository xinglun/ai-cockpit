# Ordinary Work Item

## Applicability

Implement/verify/archive/clean up; failed/stale/timed-out/invalid checks: [recovery](verification-failure-recovery.md).

## Authoritative inputs

Read Contract; query `inspect`, `status`, `doctor`, `agent doctor --json` with `--repo`. Runtime admits actions. Never edit generated records; re-query after Contract/snapshot changes.

## Before verification

Before launching a declared check, inspect `work-item status` and `work-item validate`, inventory its formal receipt, and reuse only fresh, complete evidence. Follow Runtime's admitted next action via the managed entry. See [reuse](../../docs/reference/agent-workflow.md#verification-evidence-reuse).

## Operations

Use admitted `work-item controls` or MCP `work_item_controls`; both recheck. Required controls gate finish. Material review: [commands](../../docs/reference/commands.md); Stage 1 has no opt-in.

One Work Item is serial by default; keep lifecycle, snapshot-changing, and receipt producer-consumer actions serial. Verification defaults to --workers 1. Parallelism requires Runtime-verified dependency readiness and output isolation. Independent CI jobs may fan out only with ready dependencies, isolated outputs, and bounded resources.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations, linked worktrees, registration, a current slot lease, and fresh admission; otherwise use admitted serial work or stop. Discover with `capability show`, CLI help, and MCP `tools/list`; declarations alone do not prove support; fields or an older Runtime do not prove support. If unavailable, use admitted serial work or stop. See [workflow](../../docs/reference/agent-workflow.md).

Inspection is read-only. Registration, reports, coordination, leases and drift recovery write; recovery appends events. Before dependent actions, check environment drift read-only; record changes before refreshing admission.

Amend via `work-item amend --request`, reason and current `expectedContractDigest`; `work-item amendments` reads history. Sensitive changes rerun policy and invalidate checks. Preserve identity, lifecycle, observations, failed/stale evidence and bindings. `preflight` is not verification.

## Bounded task mode

With `taskAuthorization`, query one WI with `--authorization-request` (MCP `authorizationRequest`); pass explicit `--task-caller` / `taskCaller`. Separate authority, evidence freshness and admission. Follow `nextAction`; queries are not permits. Same-scope commits require necessary revalidation, not repeated owner approval. Never infer an actor or claim human identity. After close, only preauthorized native closeout/auxiliary cleanup remains. Record completed at delivery end. [Commands](../../docs/reference/commands.md#bounded-task-authorization).

## Handoff

Deliver visible Outcome per `AGENTS.md`. Preserve unknowns; they alone do not stop admitted work. Stop for missing authority, withdrawal/expiry/completion, contradictory evidence or required human decision. Show the Runtime reason.

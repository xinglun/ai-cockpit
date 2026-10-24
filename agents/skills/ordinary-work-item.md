# Ordinary Work Item

## Applicability

Implementation/verification/archive/cleanup; failed/stale/timed-out/invalid checks: [recovery](verification-failure-recovery.md).

## Authoritative inputs

Read Contract; query `inspect`, `status`, `doctor`, `agent doctor --json` with `--repo`. Runtime admits actions. Never edit generated records; re-query after Contract/snapshot changes.

## Before verification

Before launching a declared check, inspect `work-item status` and `work-item validate`, inventory its formal receipt, and reuse only fresh, complete evidence. Otherwise follow Runtime's admitted next action. See [reuse](../../docs/reference/agent-workflow.md#verification-evidence-reuse).

## Operations

On admission use `work-item controls` or MCP `work_item_controls`; both recheck. Missing required controls block finish. Material review: [commands](../../docs/reference/commands.md); Stage 1 has no opt-in.

Keep lifecycle, snapshot-changing, and receipt producer-consumer actions serial. Verification defaults to --workers 1. Parallelism requires Runtime-verified dependency readiness and output isolation. Independent CI jobs may fan out only with ready dependencies, isolated outputs, and bounded resources.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations, linked worktrees, registration, a current slot lease, and fresh admission. Use `capability show`, help and MCP `tools/list`: declarations alone do not prove support; fields or an older Runtime do not prove support. If unavailable, use admitted serial work or stop. See [workflow](../../docs/reference/agent-workflow.md).

Inspection is read-only. Registration, reports, coordination, leases and drift recovery write; recovery appends events. Check drift read-only before dependent actions, record changes, then refresh admission.

Amend via `work-item amend --request`, reason and current `expectedContractDigest`; `work-item amendments` reads history. Sensitive changes rerun policy and invalidate checks. Preserve identity, lifecycle, observations, failed/stale evidence and bindings. `preflight` is not verification. Closeout: [commands](../../docs/reference/commands.md).

## Bounded task mode

If status has `taskAuthorization`, query one WI with `--authorization-request` (MCP `authorizationRequest`); pass explicit `--task-caller` / `taskCaller`. Separate authority, evidence freshness and actual admission. Follow `nextAction`; a query is no cached permit. Same-scope commits require necessary revalidation, not repeated owner approval. Never infer an actor or claim human identity. After close, only preauthorized native closeout/auxiliary cleanup remains. Record completed at delivery end. Fields/startup: [commands](../../docs/reference/commands.md#bounded-task-authorization).

## Handoff

Deliver visible Outcome per `AGENTS.md`. Preserve unknowns; they alone do not stop admitted work. Stop for missing authority, withdrawal/expiry/completion, contradictory evidence or required human decision. Show the Runtime reason.

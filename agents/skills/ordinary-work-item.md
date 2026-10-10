# Ordinary Work Item

## Applicability

Use for implementation, verification, archive, or cleanup. Route failed, stale, timed-out, or invalid checks to [recovery](verification-failure-recovery.md).

## Authoritative inputs

Read Contract; query `inspect`, `status`, `doctor`, and `agent doctor --json` with `--repo <repository>`. Runtime admits actions; use current help/capabilities. Never edit generated records; re-query after Contract/snapshot changes.

## Before verification

Before launching a declared check, inspect `work-item status` and `work-item validate`, inventory its formal receipt, and reuse only fresh, complete evidence. Otherwise follow Runtime's admitted next action. See [reuse](../../docs/reference/agent-workflow.md#verification-evidence-reuse).

## Operations

On admission, record controls with `work-item controls --repo <repository> --id <work-item> --input <json>` or MCP `work_item_controls`; both recheck. Refresh status; missing required controls block `finish`. Material review: [commands](../../docs/reference/commands.md); Stage 1 has no opt-in.

One Work Item is serial by default; keep lifecycle, snapshot-changing, and receipt producer-consumer actions serial. Verification defaults to --workers 1. Parallelism requires Runtime-verified dependency readiness and output isolation. Independent CI jobs may fan out only with ready dependencies, isolated outputs, and bounded resources.

Cross-Work-Item work requires supported CLI/MCP, compatible declarations, linked worktrees, registration, a current slot lease, and fresh admission; otherwise use admitted serial work or stop. Discover with `capability show`, CLI help, and MCP `tools/list`; declarations alone do not prove support; fields or an older Runtime do not prove support. If unavailable, use admitted serial work or stop. See [workflow](../../docs/reference/agent-workflow.md).

Inspection is read-only; registration, reports, coordination, leases, and drift recovery write. Recovery appends events; re-admit unrelated work. Closeout: [closeout](../../docs/reference/commands.md#cross-checkout-work-item-closeout-recovery).

Amend via `work-item amend --request` with a reason and current `expectedContractDigest`; `work-item amendments` reads append-only history. Protect identity, lifecycle, observations, and evidence; sensitive edits rerun policy and invalidate checks. Before dependent actions, check environment drift read-only; record changes before refreshing admission.

## Success conditions

Continue only when Runtime admits the action and required inputs. At handoff, deliver the human Outcome per `AGENTS.md`.

## Failure evidence

Preserve scope, history, failures, receipts, and bindings; never edit generated records. Queries are read-only; `preflight` is not verification. Report blockers and unknowns.

## Continue or stop

Preserve unknowns; unknowns alone do not stop an operation admitted by the current Runtime. Stop for missing authority, contradictory evidence, or required human decision; show the Runtime reason.

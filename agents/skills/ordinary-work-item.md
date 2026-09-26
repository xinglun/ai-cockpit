# Ordinary Work Item

Use this route when Runtime selects ordinary implementation, verification,
archive, or local cleanup. For failed/stale evidence, follow the selected
recovery guide.

## Governed work

Read the active Contract and query `inspect`, `status`, and `doctor` with an
explicit `--repo`. Runtime `safeActions`, blockers, evidence freshness, and
action explanation govern each step; this guide is not permission. Change only
Contract scope, preserve evidence/history, and re-query at lifecycle boundaries.
Run declared checks. Keep failures and invalidated receipts; never hand-edit
generated records. At handoff, give the separate human Outcome required by
`AGENTS.md`. Queries are read-only; `preflight` is an explicit, idempotent
write and does not run verification.

When fresh status recommends `record_governance_controls`, record only the
explicitly supplied evidence with `work-item controls --repo <repository>
--id <work-item> --input <json>` (or MCP `work_item_controls`). Both write
surfaces re-check current Runtime action admission. Re-query status afterward;
required controls that remain incomplete do not admit `finish`.

## Serial and cross-Work-Item use

One Work Item runs serially by default. `compatible: false` /
`parallel_compatibility_not_declared` denies only parallel work; continue
serially when Runtime admits it.

Before parallel work, refresh `inspect`/`status`, check
`ai-cockpit work-item inspect --repo <repository> --id <work-item>`, and
discover current CLI help plus MCP `tools/list` schemas. The manifest and
`agent doctor` do not prove write support. Require declared compatibility,
isolated linked worktrees in one common Git directory, current registration,
and a Runtime slot lease. If unsupported, do not emulate constraints: use
admitted serial work or stop.

Coordination inspection is read-only. Register, report impact, publish an
outcome, request/acknowledge/safely pause, resume, recover, and acquire or
relinquish leases only through their explicit Runtime write actions. Bind
records to the current repository, Contract, Runtime, and generation; reject
stale requests.
Refresh dependency admission immediately before affected actions; unrelated
actions may continue only if their own refreshed admission allows. Recovery
appends a resolution; retain the original event.

The repository-bound installed Runtime owns lifecycle decisions. Candidate
collaboration writes require tools actually implemented by that candidate;
readable new fields do not prove compatibility. See the
[agent workflow](../../docs/reference/agent-workflow.md) for the full discovery
and safe-pause sequence.

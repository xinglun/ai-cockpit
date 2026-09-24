# Cross-WI coordination capability

This feature closes the loop between dependency declaration, impact reporting,
safe-boundary coordination, exact composition verification, and human Outcome
projection. It is opt-in for a Work Item and intentionally does not turn
“request confirmation to start” into another governance gate.

The repository service is the domain boundary. CLI and MCP are adapters; the
human Outcome is a projection. Queries stay read-only, while registration,
impact, coordination transitions, recovery consumption, and composition are
explicit writes.

The supported topology is one Git common directory with linked worktrees. The
fixed Runtime owns lifecycle compatibility; the candidate Runtime must advertise
the collaboration capability before it may write or consume these records.

Registration is fact-bound: Runtime rechecks Git identity, active Contracts,
heads, branches, and regular evidence files before admission. Composition
records are shared under the Git common directory, use the bounded verification
executor, and expose real per-node reuse and cleanup facts through Outcome.

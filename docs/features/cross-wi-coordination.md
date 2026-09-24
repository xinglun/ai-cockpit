# Cross-WI coordination capability

This feature closes the loop between dependency declaration, impact reporting,
safe-boundary coordination, exact composition verification, and human Outcome
projection. It is opt-in for a Work Item and intentionally does not turn
“request confirmation to start” into another governance gate.

The repository service is the domain boundary. CLI and MCP are adapters; the
human Outcome is a projection. Queries stay read-only, while registration,
impact, outcome publication, coordination transitions, recovery consumption,
and composition are explicit writes. Outcome publication binds the current
registration generation and exact evidence bytes.

The supported topology is one Git common directory with linked worktrees. The
fixed Runtime owns lifecycle compatibility; the candidate Runtime must advertise
the collaboration capability before it may write or consume these records.

Registration is fact-bound: Runtime rechecks Git identity, active Contracts,
heads, branches, and regular evidence files before admission. Composition
records are shared under the Git common directory and use the bounded
verification executor. Reuse requires matching observed executable, command,
effective environment, declared input bytes, and dependency receipts; unknown
node inputs disable it. Outcome exposes applicability, actual merge, cleanup,
and reuse facts separately.

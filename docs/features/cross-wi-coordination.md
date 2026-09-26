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

The supported topology is one Git common directory with linked worktrees.
Installed Runtime `0.2.113` owns lifecycle compatibility and does not read the
collaboration store. The candidate Runtime must advertise the collaboration
capability before it may write or consume these records. Candidate-only Contract
check-coverage fields are not assumed to be readable or enforced by an older
installed binary.

Registration is fact-bound: Runtime rechecks Git identity, active Contracts,
heads, branches, and regular evidence files before admission. Composition
records are shared under the Git common directory and use the bounded
verification executor. Reuse requires matching observed executable, command,
effective environment, declared input bytes, and dependency receipts; unknown
node inputs disable it. Outcome exposes applicability, actual merge, cleanup,
and reuse facts separately.

An interrupted composition durably records its active verifier process group.
Retry preserves the temporary worktree while the group is alive or its state is
unknown. On Unix it also checks same-user process working directories and open
file handles under the worktree, so detached session descendants still using
the tree block cleanup; incomplete inspection fails closed. On Windows the
bounded executor's kill-on-close process job contains descendants. Outcome
reports success or reusable checks only from a coherent terminal attempt.

Composition coverage comes from digest-bound required `verification` checks
that declare `coversScenarios` or `coversConstraints`; caller labels do not
grant coverage. The execution repository must share the coordination store's
Git common directory, so an independent clone with a copied repository id is
rejected.

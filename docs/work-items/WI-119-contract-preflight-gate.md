# WI-119 — Contract preflight human-review gate

## Objective

Align the Rust Runtime's pre-edit Contract boundary with the reference Agent
workflow: uncertainty pauses implementation and asks a human instead of being
silently treated as ready.

## Scope

- Add additive Contract `sources` and `verification` declarations.
- Mark incomplete Contracts yellow with `reviewState:
  needs_human_confirmation` and persist the bound preflight receipt.
- Allow checkpoint only for green or `verification_pending` yellow; human-review
  yellow and red are fail-closed.
- Keep repository/Work Item/Contract/snapshot bindings and synchronize CLI/MCP
  projections and trilingual documentation.

## Out of scope

Release publication, global Agent/MCP configuration, and rewriting archived
Work Item bytes.

## Acceptance

1. `work-item new` followed by `preflight` is not ready and lists human fields.
2. Missing authority or Contract intent/scope/acceptance in a scaffold cannot
   cross checkpoint.
3. Missing declared verification remains `verification_pending` and may proceed
   only to collect evidence.
4. Contract and snapshot changes require a fresh preflight.
5. CLI and MCP expose the same review state, blockers, unknowns, and next
   actions.

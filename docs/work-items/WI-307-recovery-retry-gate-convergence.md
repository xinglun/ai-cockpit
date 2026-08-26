---
author: AI Cockpit maintainers
title: "Recovery retry projection gate convergence"
description: "Align the static governance-integrity gate with Runtime consumption of stale retry evidence."
audience:
  - maintainer
  - reviewer
workItemId: WI-307-recovery-retry-gate-convergence
status: in progress
lastVerifiedBy: WI-307-recovery-retry-gate-convergence
authority: canonical
---

# Recovery retry projection gate convergence

## Intent and goal

WI-306 exposed a CI-only mismatch: the Rust Runtime stops projecting a retry
receipt after fresh verification advances the predecessor Contract/Summary/
Outcome/Events bindings, but the static governance-integrity gate required a
transient blocked Summary marker and otherwise treated the retry as a current
recovered terminal state. This Work Item aligns the gate with the Runtime
identity rules without weakening fail-closed recovery handling or rewriting
historical bytes.

## Scope and source

- `tests/ci/governance_integrity_gate.py`
- `tests/ci/governance_integrity_gate_test.sh`
- `tests/ci/fixtures/governance-integrity`
- tri-language Agent workflow and command references

The source of truth is the installed Rust Runtime's read-side recovery
validation (`load_recovery_decision`) and the hosted WI-306 run
`32978852886`, which reported `docs_governance_integrity` with
`missing_parity_decision` while a fresh verification had already advanced the
archived bindings.

## Decisions

The gate consumes a `retry` only when its predecessor digest is valid and no
longer matches the fresh archived record, together with a green archived
Outcome. Minimal legacy fixtures without predecessor digests retain the
explicit blocked-Summary compatibility path. Invalid, foreign, malformed,
ambiguous, successor, and supersede records remain fail-closed. The gate then
projects the actual finalization path rather than inventing a recovered
terminal state.

No Rust Runtime protocol, repository archive, Outcome, verification, or
recovery bytes are rewritten. This is semantic alignment, not source-code or
wire-format copying.

## Acceptance and verification

- consumed stale retry with fresh green archive projects `finalize` and
  `awaiting_merge_close`;
- still-blocked retry remains a recovery boundary;
- successor/supersede and malformed/foreign candidates retain existing
  fail-closed behavior;
- tri-language workflow and command documentation state the same rule;
- run `bash tests/ci/governance_integrity_gate_test.sh`;
- run `bash tests/ci/recovery_gate_acceptance.sh`;
- run `bash tests/docs/documentation_acceptance.sh`;
- run `cargo test --locked --workspace`.

## Boundary

The external Runtime remains shared and repository state remains isolated.
This Work Item changes only the repository's static CI projection and its
documentation; it does not add provider calls, release behavior, or global
Agent/MCP configuration.

---
author: AI Cockpit maintainers
title: "WI-115 — v0.2.9 release and capability-surface parity"
description: "Publish v0.2.9 and close the reference-source command, MCP, and release-documentation parity gaps found before publication."
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-115-release-v0-2-9
capabilityClaims:
  - release_distribution
  - reference_parity
  - cli_commands
---

# WI-115 — v0.2.9 release and capability-surface parity

## Goal

Publish the next immutable Release from the reviewed default branch and make
the reference-source Agent guidance, CLI/MCP capability inventory, and release
examples truthful for future Work Items and adopters.

## Scope

- Version and release-distribution documentation for v0.2.9, including the
  three-language current baseline and N-1 example.
- Three-language MCP and CLI command inventories, including
  `delegated_evidence_list`, `capability show`, and `diagnose`.
- Three-language feature and reference-parity wording for the Runtime-generated
  validated `humanHandoff` boundary.
- Documentation acceptance checks that prevent the inventories and release
  target examples from drifting again.
- The immutable v0.2.9 Release, public adopter acceptance, and v0.2.8-to-v0.2.9
  N-1 acceptance using only downloaded public artifacts.

## Out of scope

Runtime behavior, Protocol schemas, global Agent/MCP configuration, external
Homebrew tap mutation, rewriting historical Release/evidence truth, and a
second-technology-stack adopter.

## Findings addressed

The reference comparison found that the repository had inherited the core
operating rules (one Work Item/branch/worktree/PR, explicit repository binding,
fail-closed preflight and Outcome, in-scope defect repair, immutable Release
acceptance, and no global Agent/MCP writes). It also found four documentation
drift points: one omitted MCP tool, two omitted CLI entries, an ambiguous
release target example, and an outdated statement that the Agent layer creates
the human-facing MCP projection.

## Acceptance

1. All three language capability pages enumerate the twelve tools returned by
   `tools/list`; the CLI reference enumerates `capability show` and `diagnose`.
2. Release pages identify v0.2.9 as current and use
   `x86_64-unknown-linux-gnu` for the complete adopter baseline examples; other
   targets remain explicitly additional coverage.
3. Feature and parity pages state that Runtime validates OutcomeV2 and emits
   `humanHandoff`; an Agent or conversation layer selects and displays it but
   cannot turn presentation into governance authority.
4. The documentation acceptance, version consistency, release policy, Rust
   quality, conformance, and adopter harness checks pass.
5. The public v0.2.9 artifact passes adopter and N-1 acceptance with isolated
   repository/runtime identity, cleanup, evidence reuse, and
   `first-adopter-smoke = not_ready`.
6. The Work Item completes the installed Runtime lifecycle and emits a visible
   human Outcome with 🟢/🟡/🔴 markers, unknowns, evidence, decision, and next
   action.

## Inheritance boundary

Future Work Items inherit the current `AGENTS.md`, `.ai/README.md`, and
`docs/reference/agent-workflow.*` rules. Those pages remain the repository-local
operating authority; this record is release evidence, not a replacement for
the route.

## Verification

```text
bash tests/docs/documentation_acceptance.sh
tests/release/version_consistency.sh --repo .
tests/release/action_runtime_policy.sh .github/workflows/ci.yml .github/workflows/release.yml
tests/release/workflow_policy.sh .github/workflows/release.yml
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace
tests/release/adopter_acceptance.sh --repository xinglun/ai-cockpit --tag v0.2.9 --target x86_64-unknown-linux-gnu
tests/release/adopter_upgrade_acceptance.sh --repository xinglun/ai-cockpit --from-tag v0.2.8 --to-tag v0.2.9 --target x86_64-unknown-linux-gnu
```

## Release truth

The existing v0.2.8 Release and its pre-fix/failure receipts remain immutable.
Any post-release adopter failure is recorded as failed evidence and never
rewrites the published Release truth.


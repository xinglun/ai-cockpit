---
author: AI Cockpit maintainers
title: "WI-478 — release v0.2.57 and public adopter acceptance"
description: "Publish a corrected Runtime release after the failed v0.2.56 attempt and prove the public artifact in isolation."
audience:
  - adopter
  - maintainer
  - reviewer
status: implementation_active
authority: canonical
lastVerifiedBy: WI-478-release-v0-2-57
workItemId: WI-478-release-v0-2-57
---

# WI-478 — release v0.2.57 and public adopter acceptance

[简体中文](WI-478-release-v0-2-57.zh-CN.md) · [日本語](WI-478-release-v0-2-57.ja.md)

## Intent

Publish a new immutable Runtime release using the corrected order after the
failed v0.2.56 publication. The public binary must be usable from zero in an
isolated adopter, and then be installed on this repository without changing
the local reference source or any adopter repository.

## Scope

- Align workspace package versions, lockfile entries, and current tri-language
  release/versioning guidance to `v0.2.57`, while preserving failed-release
  history and historical evidence.
- Register this Work Item in the tri-language reference-parity ledgers.
- Pass a reviewed hosted PR before creating the annotated tag; publish the
  archive, checksums, SBOM, provenance, manifest, and runtime identity.
- Run public adopter and N-1 acceptance only from downloaded public artifacts
  in isolated roots, including forbidden-write, evidence-binding, and cleanup
  proofs.
- Install the published binary on this repository and verify inspect, status,
  doctor, Agent doctor, and ready-on-base state.
- Complete verification, human Outcome, archive, resource finalization, close,
  documentation promotion, and exact branch/worktree cleanup before tagging.

## Out of scope

The local reference source, `/Users/sei-rinn/dev/workspace_rust/ai-investigation-orchestrator`,
other adopter repositories, global Agent/MCP or Homebrew configuration,
source/workspace binary fallback, unrelated Runtime redesign, and hand-editing
generated status, evidence, receipt, archive, or decision records.

## Acceptance criteria

1. Workspace package versions, lockfile entries, and required tri-language
   release documents identify `v0.2.57` without rewriting historical facts.
2. A reviewed PR passes hosted checks before merge; the annotated `v0.2.57`
   tag points exactly to the synchronized reviewed default branch and is created
   only after this Work Item is closed.
3. The public Release exposes expected archives, SHA256 checksums, SBOM,
   provenance, and an identity-bound release manifest.
4. Public adopter and N-1 acceptance use only immutable public artifacts,
   retain `first-adopter-smoke=not_ready`, bind repository/runtime identities,
   prove isolation, and prove temporary-root cleanup on success and failure.
5. The published binary is installed on this repository; inspect, status,
   doctor, Agent doctor, and documentation promotion prove healthy attachment
   and readiness.
6. The Work Item reaches a visible human Outcome with the required
   `🟢`/`🟡`/`🔴` marker, then archive/finalization/close and exact cleanup.

## Verification

```text
cargo test --locked --workspace
```

Release publication and public acceptance are post-release evidence. A failed
publication remains immutable failed history and is never relabeled or reused.

## Boundary

The installed Runtime is shared, while this repository's Protocol, Work Items,
evidence, knowledge, and adapters remain private. Publishing the Runtime never
implicitly attaches or mutates a target repository.

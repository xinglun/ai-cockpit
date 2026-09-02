---
author: AI Cockpit maintainers
title: "WI-509 — release v0.2.61 and public adopter acceptance"
description: "Publish the next identity-bound Runtime release and prove the public artifact before resuming reference parity work."
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-509-release-v0-2-61
terminalArchive: .ai/work-items/archive/WI-509-release-v0-2-61.contract.json
terminalVerification: .ai/evidence/WI-509-release-v0-2-61.verification.json
terminalFinalization: .ai/decisions/WI-509-release-v0-2-61.finalize.json
terminalDecision: .ai/decisions/WI-509-release-v0-2-61.close.json
workItemId: WI-509-release-v0-2-61
---

# WI-509 — release v0.2.61 and public adopter acceptance

[简体中文](WI-509-release-v0-2-61.zh-CN.md) · [日本語](WI-509-release-v0-2-61.ja.md)

## Intent

Publish v0.2.61 from reviewed main and prove the immutable public release
artifact with isolated adopter acceptance before resuming reference parity.

## Scope

- Align workspace package versions, lockfile entries, and current tri-language
  release/versioning guidance to v0.2.61 while preserving historical facts.
- Register this release Work Item in the tri-language reference-parity ledgers.
- Pass a reviewed hosted PR before creating the annotated tag from synchronized
  main; publish archives, checksums, SBOM, provenance, and manifest.
- Run public adopter and N-1 acceptance only from downloaded immutable
  artifacts, including isolation, evidence binding, not_ready scaffold, and
  cleanup proofs.
- Install the published binary on this repository and verify inspect, status,
  doctor, Agent doctor, and documentation-promotion health.

## Out of scope

The local reference source, the object/adopter repository, global Agent/MCP or
Homebrew configuration, source/workspace binary fallback, unrelated Runtime
redesign, and hand-editing generated governance records.

## Acceptance criteria

1. Workspace packages and lockfile identify v0.2.61; current tri-language
   release guidance is updated without rewriting prior release history.
2. Hosted checks pass on the reviewed PR before the annotated v0.2.61 tag is
   created from synchronized main.
3. The public Release exposes identity-bound archives, SHA256 checksums, SBOM,
   provenance, and release manifest.
4. Public adopter and N-1 acceptance use only downloaded immutable artifacts,
   prove isolation and temporary-root cleanup, and preserve
   first-adopter-smoke=not_ready.
5. The published binary is installed on this repository; inspect, status,
   doctor, Agent doctor, and post-close documentation checks remain healthy.
6. This Work Item has a visible human Outcome, archive, finalization, close,
   and exact branch/worktree cleanup before publication.

## Verification

```text
cargo test --locked --workspace
```

Release publication and public acceptance are post-release evidence. A failed
publication remains immutable failed history and is never relabeled or reused.

## Boundary

The Runtime binary is shared, while this repository's Protocol, Work Items,
evidence, knowledge, and adapters remain repository-local. Publishing the
Runtime never implicitly attaches or mutates another repository.

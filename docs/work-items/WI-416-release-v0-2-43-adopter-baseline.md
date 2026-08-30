---
author: AI Cockpit maintainers
title: WI-416 — v0.2.43 post-release adopter baseline
description: Preserve the public v0.2.43 adopter acceptance receipt and its runtime identity.
workItemId: WI-416-release-v0-2-43-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-416-release-v0-2-43-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-416-release-v0-2-43-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-416-release-v0-2-43-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.finalize.json
terminalDecision: .ai/decisions/WI-416-release-v0-2-43-adopter-baseline.close.json
---

# WI-416 — v0.2.43 post-release adopter baseline

[简体中文](WI-416-release-v0-2-43-adopter-baseline.zh-CN.md) · [日本語](WI-416-release-v0-2-43-adopter-baseline.ja.md)

## Intent

Persist a reproducible post-release acceptance baseline for the public
v0.2.43 Release. The evidence binds the downloaded archive and binary to the
release identity, adopter repository identity, lifecycle, isolation, and
temporary-run cleanup.

## Evidence boundary

The complete harness output is retained under
`.ai/evidence/WI-416-release-v0-2-43-adopter-acceptance/`. It includes
`runtime.json` (archive digest and binary SHA-256), `repository.json`, attach,
profile and Agent doctor outputs, `first-adopter-smoke` with `state: not_ready`,
verification reuse, lifecycle records, isolation manifests, `cleanup.json`,
`acceptance.json`, and `SHA256SUMS`. The harness used only the immutable public
v0.2.43 archive and removed its temporary run root.

## Acceptance

- `acceptance.json` reports `releasePublished: true` and
  `adopterAcceptance: passed`; every recorded step is passed and cleanup is
  validated.
- The lifecycle verification evidence is schema 2, binds the adopter
  `repositoryId`, and records Runtime `0.2.43` with binary digest
  `sha256:d6334275904868d7e7e46a569e4198d75057d25f22997781df1a7097a3e70533`.
- The persisted checksum file validates every retained artifact; historical
  receipts remain unchanged.

## Non-claims

This is a public-artifact acceptance record, not a new Runtime governance
authority, a source build, or a second technology-stack acceptance.

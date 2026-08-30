---
author: AI Cockpit maintainers
title: WI-419 — v0.2.44 post-release adopter baseline
description: Preserve the public v0.2.44 adopter acceptance receipt and runtime identity.
workItemId: WI-419-release-v0-2-44-adopter-baseline
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-419-release-v0-2-44-adopter-baseline
terminalArchive: .ai/work-items/archive/WI-419-release-v0-2-44-adopter-baseline.contract.json
terminalVerification: .ai/evidence/WI-419-release-v0-2-44-adopter-baseline.verification.json
terminalFinalization: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.finalize.5e69364aa22b2a2fa6dafd2af75cd5eef1cc6b31b01bd41c09f4cdad956e9a08.json
terminalDecision: .ai/decisions/WI-419-release-v0-2-44-adopter-baseline.close.json
---

# WI-419 — v0.2.44 post-release adopter baseline

[简体中文](WI-419-release-v0-2-44-adopter-baseline.zh-CN.md) · [日本語](WI-419-release-v0-2-44-adopter-baseline.ja.md)

## Intent

Persist a reproducible post-release acceptance baseline for the public
v0.2.44 Release. The receipt binds the downloaded archive and binary to the
release identity, adopter repository identity, lifecycle, isolation, evidence
reuse, and temporary-run cleanup.

## Evidence boundary

The complete public-binary harness output is retained under
`.ai/evidence/WI-419-release-v0-2-44-adopter-acceptance/`. It includes
`runtime.json` (archive and binary SHA-256), `repository.json`, attach,
profile and Agent doctor outputs, `first-adopter-smoke` with `state: not_ready`,
verification reuse, complete Work Item lifecycle records, isolation manifests,
`cleanup.json`, `acceptance.json`, and `SHA256SUMS`. The run downloaded only the
immutable public v0.2.44 archive for `aarch64-apple-darwin`; its temporary run
root was validated as removed.

## Acceptance

- `acceptance.json` reports `releasePublished: true` and
  `adopterAcceptance: passed`; every recorded step and cleanup validation pass.
- Runtime identity is version `0.2.44`, binary digest
  `sha256:69d28c970c2b89534e63cb685c6cc02a2f135d3067b6a84feaabce2adce1d5e5`,
  and the adopter repository identity is
  `sha256:26301b33fabbb72aaacb48c8f9ccac335be8ca5aa42b9e98941324d2108a8df1`.
- The lifecycle verification evidence is schema 2, reuses prior evidence with
  zero new process spawns, and records a structured close decision. The
  persisted checksum file validates every retained artifact.

## Non-claims

This is a public-artifact acceptance record, not new Runtime governance
authority, a source build, a V1 fixture, or a second technology-stack
acceptance. Historical receipts remain immutable.

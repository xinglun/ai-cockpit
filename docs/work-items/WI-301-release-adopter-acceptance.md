---
author: AI Cockpit maintainers
title: "WI-301 — v0.2.33 public Release adopter acceptance"
workItemId: WI-301-release-adopter-acceptance
description: "Validate the immutable v0.2.33 binary in a fresh isolated adopter and verify a public N-1 upgrade."
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
lastVerifiedBy: WI-301-release-adopter-acceptance
terminalArchive: .ai/work-items/archive/WI-301-release-adopter-acceptance.contract.json
terminalVerification: .ai/evidence/WI-301-release-adopter-acceptance.verification.json
terminalFinalization: .ai/decisions/WI-301-release-adopter-acceptance.finalize.json
terminalDecision: .ai/decisions/WI-301-release-adopter-acceptance.close.json
authority: canonical
---

# WI-301 — v0.2.33 public Release adopter acceptance

## Intent

Prove that the immutable, publicly published v0.2.33 Release binary can govern
a new repository from zero, and that a repository created by the published
v0.2.31 binary can upgrade without losing its historical evidence.

## Scope

This acceptance uses only downloaded public Release artifacts on
`aarch64-apple-darwin`. It records the archive and executable SHA-256,
repository identities, Runtime identity, attach/profile/Agent doctor output,
the `first-adopter-smoke` `not_ready` contract skeleton, evidence reuse, the
complete Work Item lifecycle, N-1 upgrade history preservation, isolation
manifests, and temporary-root cleanup.

The receipts are retained under:

- `.ai/evidence/external/v0.2.33/adopter-aarch64-apple-darwin/`
- `.ai/evidence/external/v0.2.33/upgrade-v0.2.31-to-v0.2.33/`

## Evidence boundary

`runtime.json` binds tag `v0.2.33`, archive digest
`sha256:c8019db3d8509d62418afed114b986689df7b0ef570ff7199a4b845c7d932ca4`,
and extracted binary digest
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.
The upgrade receipt binds the public N-1 tag `v0.2.31` to v0.2.33 and preserves
the old evidence bytes byte-for-byte. `acceptance.json` reports
`releasePublished: true`, `adopterAcceptance: passed`, and `cleanupState:
passed`; a failed post-release run would remain failed evidence and could not
rewrite Release truth.

HOME and XDG_CONFIG_HOME are forbidden-write roots. TMPDIR and CARGO_HOME are
explicitly isolated Runtime-write roots. The cleanup receipt proves that each
validated temporary `run_root` was removed, including failure-safe paths.

The acceptance harness is post-release evidence. A second technology-stack
adopter remains a separate Work Item; this record does not claim it.

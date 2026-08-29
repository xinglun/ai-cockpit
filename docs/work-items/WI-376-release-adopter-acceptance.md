---
author: AI Cockpit maintainers
title: "WI-376 — v0.2.39 release adopter acceptance"
description: "Validate the immutable public Release against the current repository and a fresh independent adopter."
workItemId: WI-376-release-adopter-acceptance
audience: [maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-376-release-adopter-acceptance
terminalArchive: .ai/work-items/archive/WI-376-release-adopter-acceptance.contract.json
terminalVerification: .ai/evidence/WI-376-release-adopter-acceptance.verification.json
terminalFinalization: .ai/decisions/WI-376-release-adopter-acceptance.finalize.json
terminalDecision: .ai/decisions/WI-376-release-adopter-acceptance.close.json
capabilityClaims: [release_acceptance, repository_isolation, evidence_reuse]
---

# WI-376 — v0.2.39 release adopter acceptance

[简体中文](WI-376-release-adopter-acceptance.zh-CN.md) · [日本語](WI-376-release-adopter-acceptance.ja.md)

## Intent

Prove that the immutable public v0.2.39 Release can govern this repository and
a fresh independent adopter without sharing repository state or relying on a
source checkout.

## Scope and boundary

- Verify the downloaded public archive, binary digest, manifest, and checksums.
- Verify v0.2.39 Runtime inheritance in the current repository.
- Attach a fresh adopter, inspect its scaffold, and exercise a complete Work
  Item lifecycle with valid evidence and a structured close decision.
- Prove exact evidence reuse, changed-snapshot re-execution, and global-root
  isolation; preserve an auditable acceptance receipt and then clean temporary
  state.

New Runtime features, source or workspace binary fallback, a second technology
stack, and global Agent/MCP configuration are outside this Work Item.

## Acceptance

1. The downloaded v0.2.39 archive and binary match `release-manifest.json` and
   `SHA256SUMS`.
2. The current repository is `COMPATIBLE` and `ready_on_base`; `doctor` is ok,
   `runtimeCodeInRepository` is false, and Agent doctor is `VERIFIED`.
3. A fresh adopter has a distinct `repositoryId` and receives only the minimum
   repository scaffold.
4. Its new Work Item skeleton is `not_ready`; human intent, scope, acceptance,
   and authority are not invented by the Runtime.
5. The adopter lifecycle emits schema-2 evidence bound to repository, snapshot,
   Work Item, Runtime identity, and close decision.
6. An exact repeated verification reuses evidence without execution; a changed
   snapshot re-executes verification.
7. Forbidden global roots remain unchanged and runtime writes stay in isolated
   roots.
8. Acceptance artifacts include runtime identity, JSON outputs, reuse and
   isolation proofs, lifecycle evidence, and checksums; temporary adopter and
   run roots are removed afterward.

## Verification boundary

The published Release is the only Runtime under test. Acceptance receipts are
post-release evidence and do not alter the immutable Release truth.

## Result

The v0.2.39 public archive and binary were verified against the release
manifest and checksums. The current repository inherited Runtime 0.2.39 with
healthy `inspect`, `status`, `doctor`, and Agent doctor results. A fresh
independent adopter received a distinct repository identity, preserved a
`first-adopter-smoke` skeleton as `not_ready`, and completed a schema-2
verification/finish/archive/finalize/close lifecycle. Exact release-artifact
and adopter receipts are stored under
`release-adopter-acceptance-artifacts/`; the fixed adopter path and isolated
retry roots were removed after capture.

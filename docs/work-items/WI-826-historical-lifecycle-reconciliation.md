---
author: AI Cockpit maintainers
title: "WI-826 — historical evidence archive compatibility"
description: "Provide an explicit, fail-closed archive route for typed evidence captured by an older Runtime without rewriting historical bytes."
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorized
workItemId: WI-826-historical-lifecycle-reconciliation
lastVerifiedBy: WI-826-historical-lifecycle-reconciliation
terminalArchive: .ai/work-items/archive/WI-826-historical-lifecycle-reconciliation.contract.json
terminalVerification: .ai/evidence/WI-826-historical-lifecycle-reconciliation.verification.json
terminalDecision: .ai/decisions/WI-826-historical-lifecycle-reconciliation.close.json
---

[简体中文](WI-826-historical-lifecycle-reconciliation.zh-CN.md) · [日本語](WI-826-historical-lifecycle-reconciliation.ja.md)

# WI-826 — historical evidence archive compatibility

## Intent and boundary

This Work Item adds an explicit historical-evidence archive route for typed
verification evidence captured by an older Runtime. It preserves the original
evidence bytes and binds the archive manifest to their raw digest, Work Item,
repository, Contract, snapshot, Runtime, and receipt identities. Ordinary
current-Runtime archive behavior remains unchanged.

## Fail-closed behavior

Historical archival rejects current-Runtime evidence, malformed or legacy
untyped evidence, digest tampering, and identity mismatches without writing a
partial archive. The archive validator rechecks the bound evidence file and
all recorded identities; the compatibility route is explicit and does not
grant a general historical exemption.

## Acceptance evidence

- Archive: `.ai/work-items/archive/WI-826-historical-lifecycle-reconciliation.contract.json`
- Formal verification: `.ai/evidence/WI-826-historical-lifecycle-reconciliation.verification.json`
- Close decision: `.ai/decisions/WI-826-historical-lifecycle-reconciliation.close.json`

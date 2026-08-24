---
author: AI Cockpit maintainers
title: "WI-240 — Documentation status and reference truth consistency"
workItemId: WI-240-doc-status-consistency
description: "Bind Work Item status, reference inventory, parity, and release claims to current repository evidence without rewriting historical governance bytes."
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-240-doc-status-consistency
authority: canonical
---

# WI-240 — Documentation status and reference truth consistency

This Work Item refreshes documentation truth at the v0.2.31 Runtime and
`origin/main` comparison baseline. It does not reinterpret or rewrite archived
Contracts, evidence, decisions, or published Release records.

## Acceptance boundary

- The reference inventory binds target commit
  `1c988ce9b04c3dcd45843f6577ed321457eeca0e`, ignores checkout-only drift, and
  preserves exactly four capability/profile `migrate-gap` records plus 720
  `deferred-next-batch` records.
- English, Simplified Chinese, and Japanese Work Item documents agree on
  identity, projected status, and verifier. Terminal projections require a
  repository-bound archived Contract plus close or recovery evidence; ambiguous
  cross-document verifier semantics remain unknown rather than guessed.
- Historical recovery admits evidence-bound display projections: `Recovered`
  may be `historical` or `recovered`, while `Implemented` may be `recovered`
  only when the document explicitly identifies immutable recovery history.
- Release documentation records v0.2.31 as identity-bound and drift-detectable
  because the provider reports `immutable: false`. The repository-persisted
  adopter baseline is `aarch64-apple-darwin`; hosted Linux workflow artifacts
  remain short-lived external evidence.

## Evidence

The deterministic inventory, documentation acceptance, and Work Item status
regressions are bound by
`.ai/evidence/WI-240-doc-status-consistency.verification.json` and the archived
Work Item manifest. The four unresolved file-level gaps remain visible in the
machine-readable inventory and are not closed by documentation projection.

## References

- [Reference file comparison](../reference/reference-file-comparison.md)
- [Reference source parity](../reference/reference-parity.md)
- [Release distribution](../release/distribution.md)

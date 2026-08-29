---
author: AI Cockpit maintainers
title: "WI-387 — reference documentation batch 20"
workItemId: WI-387-reference-documentation-batch-20
description: "Compare four pinned security and supply-chain documents and record bounded Rust-native parity without copying source authority."
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-387-reference-documentation-batch-20
---

# WI-387 — reference documentation batch 20

## Intent and boundary

Compare the four deferred security and supply-chain documents at pinned source
commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`, then record one bounded
decision per file in the inventory and all three parity ledgers.

This is semantic/documentation parity, not source command, JSON-wire, or
provider-state compatibility. The Rust-native target may reject or stop
repository actions that conflict with declared governance facts, but it is not
a general prompt-injection detector. Supply-chain provenance, signatures,
SBOMs, vulnerability results, and trust roots remain delegated external
evidence. Do not copy source Python, Make, provider configuration, or
historical evidence as current authority.

## File decisions

| Pinned path | Decision | Maintained target boundary |
| --- | --- | --- |
| `docs/security/injection-boundary.ja.md` | `implemented-different-by-design` | Japanese `adversarial-validation`, `input-trust-dataflow`, and `operation-time-policy-reevaluation` preserve bounded injection handling, fail-closed reevaluation, and external-control limits. |
| `docs/security/injection-boundary.md` | `implemented-different-by-design` | Rust-native security/trust-flow docs preserve repository-governance boundaries; untrusted text remains data and no general detector claim is made. |
| `docs/security/injection-boundary.zh-CN.md` | `implemented-different-by-design` | Chinese Rust-native security/trust-flow docs preserve deterministic fail-closed handling and explicit non-claims. |
| `docs/security/supply-chain.md` | `implemented-different-by-design` | Threat-model, CI-release-evidence, distribution, and security-release-verification docs preserve delegated evidence ownership and exact artifact binding; Runtime does not generate external assurance. |

## Acceptance

- All four pinned source files are read and have one inventory classification,
  explicit Rust-native counterparts, and a bounded reason; `migrate-gap`
  remains zero.
- English, Chinese, and Japanese comparison/parity ledgers describe the same
  four decisions and updated counts (`4262/298/1/4/47/507/0`).
- Injection and supply-chain boundaries distinguish local governance evidence
  from external provider/security controls, without copying source commands or
  historical claims.
- Every attached object/adopter repository inherits the same Rust-native
  documentation boundary through the shared Runtime, while repository facts,
  Work Items, evidence, and snapshots remain isolated by explicit `--repo`.
- Documentation, inventory, governance, and installed Runtime lifecycle checks
  pass; no unrelated Runtime code or historical evidence is modified.

## Verification

Declared checks include the reference inventory documentation and shell tests,
documentation/status consistency, governance integrity gate, and the installed
Runtime `inspect`, `status`, `doctor`, `preflight`, `checkpoint`, `verify`,
`finish`, `archive`, and `close` lifecycle with explicit repository context.

[简体中文](WI-387-reference-documentation-batch-20.zh-CN.md) · [日本語](WI-387-reference-documentation-batch-20.ja.md)

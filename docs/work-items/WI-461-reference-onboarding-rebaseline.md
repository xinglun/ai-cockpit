---
author: AI Cockpit maintainers
title: "WI-461 — getting-started onboarding rebaseline"
workItemId: WI-461-reference-onboarding-rebaseline
description: "Re-read the nine changed local-reference onboarding pages and close their semantic inventory decisions."
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-461-reference-onboarding-rebaseline
terminalArchive: .ai/work-items/archive/WI-461-reference-onboarding-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-461-reference-onboarding-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-461-reference-onboarding-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-461-reference-onboarding-rebaseline.close.json
---

# WI-461 — getting-started onboarding rebaseline

This Work Item re-reads the nine onboarding pages whose source bytes changed
between the historical comparison commit `e5acb677da6621004d96f0ef353c58fe8d3acfbf`
and the maintained local reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060`:
`/Users/sei-rinn/dev/workspace_python/ai-cockpit-template`. The source checkout
is local and pinned; no public reference or source implementation is copied.

[简体中文](WI-461-reference-onboarding-rebaseline.zh-CN.md) · [日本語](WI-461-reference-onboarding-rebaseline.ja.md)

## File-level decisions

| Pinned reference path | Classification | Rust-native counterpart and boundary |
| --- | --- | --- |
| `docs/getting-started/first-work-item.md` | `implemented-different-by-design` | The Rust page keeps the complete repository-bound start → preflight → checkpoint → verify → finish → archive → reviewed merge → cleanup → close route, visible human Outcome, and human-review stop. The source-only Make command and removed `REPORT_LANGUAGE` argument are not copied. |
| `docs/getting-started/first-work-item.zh-CN.md` | `implemented-different-by-design` | The Chinese page preserves the same lifecycle and stop conditions with explicit `--repo`; language presentation does not alter Contract facts. |
| `docs/getting-started/first-work-item.ja.md` | `implemented-different-by-design` | The Japanese page preserves the same lifecycle, provider-resource boundary, and exact cleanup path; its duplicate merge paragraph was corrected in this batch. |
| `docs/getting-started/security-release-verification.md` | `implemented-different-by-design` | The Rust release/distribution and installation-security pages preserve tag, digest, SBOM, provenance, provider-responsibility, and adopter-isolation boundaries using the current manifest/SHA256SUMS route. The source `release.json` projection is not copied. |
| `docs/getting-started/security-release-verification.zh-CN.md` | `implemented-different-by-design` | The Chinese release route keeps the same evidence separation and fail-closed mismatch rule through Rust-native release assets and external-provider boundaries. |
| `docs/getting-started/security-release-verification.ja.md` | `implemented-different-by-design` | The Japanese release route keeps the same digest, provenance, SBOM, and public-adopter limits without importing source installer behavior. |
| `docs/getting-started/standard-adoption-guide.md` | `implemented-different-by-design` | The Rust guide retains reader-first install, attach, calibration, adapter, Work Item, Outcome, merge, cleanup, and close stages with shared Runtime semantics; source Make workflow bytes are not a target contract. |
| `docs/getting-started/standard-adoption-guide.zh-CN.md` | `implemented-different-by-design` | The Chinese guide preserves the ordered adoption boundaries and explicit repository ownership with Rust CLI routes. |
| `docs/getting-started/standard-adoption-guide.ja.md` | `implemented-different-by-design` | The Japanese guide preserves the same ordered adoption route and shared Runtime boundary without source-specific commands. |

This is semantic/documentation parity, not source-file or JSON-wire parity. The
target deliberately uses one shared installed Runtime and explicit `--repo`;
adopter projects inherit the governance boundary while keeping repository state
and provider configuration isolated.

## Verification boundary

The inventory keeps `sourceChangedSincePrevious`, `previousBatch`, and
`previousClassification` as immutable comparison provenance while promoting the
nine records from `deferred-next-batch` to an evidence-backed result. The batch
must pass the offline source policy, inventory checks, tri-language getting-
started semantic checks, documentation acceptance, governance-integrity checks,
and the declared locked Rust verification command. No runtime behavior or
object/adopter repository is changed by this documentation-only batch.

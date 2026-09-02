---
author: AI Cockpit maintainers
title: Test architecture
description: Layered, negative-first verification and quality-gate ownership in the Rust project.
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - layered_verification
---

# Test architecture

[English](test-architecture.md) · [简体中文](test-architecture.zh-CN.md) · [日本語](test-architecture.ja.md)

Verification is layered and negative-first. A layer is reported as verified
only when the repository contains explicit evidence for it; an unavailable
layer is `not_applicable` or `unknown`, never silently green.

| Layer | Rust evidence boundary |
| --- | --- |
| Protocol/schema/state machine | `cargo test --workspace`, typed protocol tests, lifecycle and property regressions |
| Repository transaction and lifecycle | repository/CLI integration tests for attach, Contract, checkpoint, verify, finish, archive, close, recovery, and isolation |
| Verification executor | bounded argv execution, worker limits, reuse identity, failure retention, and scope tests |
| Security/adversarial | conformance and absurd-case fixtures, path/symlink/identity tampering, prompt-injection and weakening regressions |
| Hosted platform | GitHub Actions Windows/runtime and V1 semantic oracle checks; provider state remains external evidence |
| Release/adopter | immutable public archive, checksum/SBOM/provenance, fresh adopter and N-1 upgrade harnesses |
| Documentation/governance | tri-language metadata, parity, inventory, status-promotion, and governance-integrity checks |

The dynamic quality route selects `light`, `standard`, or `strict` from changed
surfaces, Contract policy, and stage. The route is a Verification strength,
not an Evidence Assurance claim. A lower-cost route may omit unrelated layers,
but it cannot lower mandatory floors or turn an unknown into a pass.

Local checks prove repository facts only. They do not prove provider approval,
enterprise identity, complete external-consumer compatibility, or universal
test coverage. Reports under `target/` and generated receipts are evidence
outputs and must not be hand-edited.

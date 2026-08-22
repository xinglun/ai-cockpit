---
author: AI Cockpit maintainers
title: "Operations"
description: "Operate, verify, recover, upgrade, and accept an AI Cockpit repository."
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_operations
---

# Operations

- Follow [Capabilities and boundaries](../capabilities.md) for the governed Work Item sequence and stop conditions.
- Use [Reference](../reference/README.md) for exact command and recovery details.
- Use [Release and distribution](../release/distribution.md) for immutable Release verification, upgrade, rollback, and post-release adopter acceptance.
- Use [Versioning](../architecture/versioning.md) to distinguish a shared Runtime upgrade from an explicit repository migration.
- Use [Performance acceptance](../../tests/performance/README.md) and [Adversarial validation](../security/adversarial-validation.md) for measured or negative evidence.

The current public adopter acceptance baseline is complete only for
`x86_64-unknown-linux-gnu`; the release consistency gate resolves the baseline
version from Cargo workspace metadata. Other release targets have build or smoke
evidence unless a separate acceptance run is recorded. Legacy evidence remains
historical and must not be promoted to fresh green verification.

For MCP result delivery, use `work_item_outcome` for the visible human handoff
and retain `work_item_get` for machine lookup. Release adopter receipts must
include the typed isolation manifests and cleanup proof; allowed temporary
writes are limited to the explicitly isolated TMPDIR and CARGO_HOME roots.

[Current route](../current/README.md) | [Getting started](../getting-started/README.md) |
[中文](README.zh-CN.md) | [日本語](README.ja.md)

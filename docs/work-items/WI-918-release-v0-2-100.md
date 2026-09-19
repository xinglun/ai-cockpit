---
author: AI Cockpit maintainers
workItemId: WI-918-release-v0-2-100
title: Final release v0.2.100 after Outcome language, HCI, four-direction, and Issue #851 closure
description: Publish the reviewed main line only after all prerequisite Work Items and issues are complete.
audience: [maintainer, reviewer, adopter]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-918-release-v0-2-100
terminalArchive: .ai/work-items/archive/WI-918-release-v0-2-100.contract.json
terminalVerification: .ai/evidence/WI-918-release-v0-2-100.verification.json
terminalFinalization: .ai/decisions/WI-918-release-v0-2-100.finalize.json
terminalDecision: .ai/decisions/WI-918-release-v0-2-100.close.json
---

[简体中文](WI-918-release-v0-2-100.zh-CN.md) · [日本語](WI-918-release-v0-2-100.ja.md)

# WI-918 — Final release v0.2.100

This release route follows the completed Outcome language, HCI, four-direction,
Issue #851, and Rust/toolchain work. It publishes only the reviewed main line
and independently verifies downloaded release artifacts.

Object repositories are out of scope. Performance benefit and host display
confirmation remain explicitly unknown unless direct evidence proves otherwise.

## Acceptance

- Workspace versions, lock metadata, and current English, Simplified Chinese,
  and Japanese release/reference projections identify v0.2.100 consistently.
- The annotated v0.2.100 tag is immutable and bound to the reviewed source
  commit.
- The public Release binds manifest, checksums, SBOM, archives, attestation,
  source identity, and workflow handoff.
- Downloaded assets pass checksum, isolated fresh-install, v0.2.99 upgrade,
  and exact temporary-root cleanup without changing object repositories.
- The final Outcome follows the current conversation language and preserves
  unknown performance benefit and host display confirmation.

## Verification plan

Run cheap format, version, documentation, parity, governance, and workspace
checks before dispatching the release workflow. Consume the publication
handoff, perform downloaded adopter acceptance, preserve immutable evidence,
and close only after exact provider and branch/worktree cleanup are verified.

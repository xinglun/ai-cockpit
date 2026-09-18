---
author: AI Cockpit maintainers
workItemId: WI-911-release-v0-2-99
title: Final release v0.2.99 after Outcome language, HCI, four-direction, and Issue #851 closure
description: Publish the reviewed main line only after all prerequisite Work Items and issues are complete.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-911-release-v0-2-99
---

[简体中文](WI-911-release-v0-2-99.zh-CN.md) · [日本語](WI-911-release-v0-2-99.ja.md)

# WI-911 — Final release v0.2.99

## Intent and boundary

This release Work Item is the final publication route after Outcome language
selection, HCI correction, four-direction convergence, Issue #851, and the
Rust/toolchain update have been completed and merged. It updates the release
identity and proves the published artifact independently of source checkout.

Object repositories are explicitly out of scope. This Work Item does not add
new Outcome, host-display, performance, lifecycle, or governance behavior.

## Acceptance

- Workspace package versions, lock metadata, current release/reference
  documentation, and generated archives identify v0.2.99 consistently.
- The annotated v0.2.99 tag is immutable and bound to the reviewed source
  commit; existing tags remain unchanged.
- The provider Release is stable and bound to its manifest, checksums, SBOM,
  archives, attestation, source identity, and workflow handoff.
- Downloaded v0.2.99 assets pass checksum, isolated fresh-install, and v0.2.98
  upgrade acceptance with exact temporary-root cleanup; object repositories are
  unchanged.
- English, Simplified Chinese, and Japanese projections are synchronized, and
  the final Outcome follows the conversation language while preserving unknown
  performance benefit and host display confirmation.

## Verification plan

Run format, version, documentation, parity, governance, and workspace checks
before expensive release jobs. Then use the reviewed main line and dispatch-only
release workflow, preserve failed-candidate evidence, consume the publication
handoff, perform downloaded adopter acceptance, and close only after exact
provider and branch/worktree cleanup evidence is bound.

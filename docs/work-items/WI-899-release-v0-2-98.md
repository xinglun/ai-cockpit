---
author: AI Cockpit maintainers
workItemId: WI-899-release-v0-2-98
title: Final release v0.2.98 after Outcome, HCI, four-direction, and Issue #851 closure
description: Publish the reviewed main line only after all prerequisite Work Items and issues are complete.
audience: [maintainer, reviewer, adopter]
status: implemented
authority: user:release-after-all-work-items
lastVerifiedBy: WI-899-release-v0-2-98
---

[简体中文](WI-899-release-v0-2-98.zh-CN.md) · [日本語](WI-899-release-v0-2-98.ja.md)

# WI-899 — Final release v0.2.98

## Intent and boundary

This release Work Item is the final publication route after the Outcome
delivery work, HCI correction, four-direction convergence work, Issue #851, and
the Rust/toolchain update have been completed and merged. It updates the
release identity and proves the published artifact independently of source
checkout state.

Object repositories are explicitly out of scope. This Work Item does not add
new Outcome, host-display, performance, lifecycle, or governance behavior.

## Acceptance

- Workspace package versions, lock metadata, current release documentation,
  and generated archives identify v0.2.98 consistently.
- The annotated v0.2.98 tag is immutable and bound to the reviewed source
  commit; existing tags remain unchanged.
- The provider Release is stable and bound to its manifest, checksums, SBOM,
  archives, source identity, and workflow handoff.
- Downloaded v0.2.98 assets pass checksum, isolated fresh-install, and v0.2.97
  upgrade acceptance with exact temporary-root cleanup.
- English, Simplified Chinese, and Japanese release/reference projections are
  synchronized, and the final Outcome reports facts, limits, and next action.

## Verification plan

Run format, version, and contract prechecks before expensive release jobs. Then
use the reviewed PR and dispatch-only release workflow, preserve any failed
candidate evidence, consume the publication handoff, perform downloaded
adopter acceptance, and close only after exact branch/worktree cleanup and
post-release evidence are bound.

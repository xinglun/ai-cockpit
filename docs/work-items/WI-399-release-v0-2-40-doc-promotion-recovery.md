---
author: AI Cockpit maintainers
title: "WI-399 — v0.2.40 documentation-promotion recovery"
description: "Recover the WI-398 delivery in a dedicated worktree and preserve an auditable release baseline."
workItemId: WI-399-release-v0-2-40-doc-promotion-recovery
audience: [maintainer, reviewer]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-399-release-v0-2-40-doc-promotion-recovery
capabilityClaims: [documentation_governance, release_distribution]
---

# WI-399 — v0.2.40 documentation-promotion recovery

[简体中文](WI-399-release-v0-2-40-doc-promotion-recovery.zh-CN.md) · [日本語](WI-399-release-v0-2-40-doc-promotion-recovery.ja.md)

## Intent

Recover the WI-398 delivery after its finalization attempt correctly refused
the main worktree. The successor preserves the immutable WI-398 archive and
recovery decision, and performs the delivery from a dedicated worktree.

## Boundary

This Work Item covers only the recovery decision, the three-language WI-399
documentation, and the reference-parity registration. It does not change
Runtime semantics, release implementation, public adopter acceptance, or
historical evidence bytes.

## Verification and delivery

Documentation acceptance, repository governance checks, and the full locked
workspace test suite must pass before archive. The branch and dedicated
worktree are removed only after the reviewed PR is merged; finalization and
close then record the exact cleanup and successor linkage.

---
author: AI Cockpit maintainers
title: "Standard adoption guide"
description: "The complete reader-first route from verified Runtime to the first closed Work Item."
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Standard adoption guide

Use these stages in order. Each stage has a separate evidence boundary:

1. [Install](installation.md) an immutable public Runtime and verify the exact artifact.
2. [Inspect and attach](30-second-start.md) the intended repository explicitly.
3. Complete [first calibration](first-calibration.md) with one owner-approved quality command.
4. Complete the [adopter configuration](adopter-configuration.md) checklist for review, security, recovery, and CI ownership.
5. If needed, install one repository-local Agent adapter explicitly and verify it with `agent doctor`.
6. Run the [first Work Item](first-work-item.md) on a dedicated branch/worktree and reviewed PR.
7. Surface the human Outcome before archive; after merge, verify exact resource cleanup and record the structured human close decision.

Do not collapse installation, attachment, profile confirmation, implementation,
provider review, and close into one implied approval. A pass at one boundary
does not prove the next. Unknown or contradictory evidence stops only the claim
that depends on it and names the owner and recovery condition.

For release trust and private-mirror limits, read
[Security and release verification](security-release-verification.md). Platform
examples show how to preserve Unknown without inventing project facts.

[Getting started](README.md) | [中文](standard-adoption-guide.zh-CN.md) | [日本語](standard-adoption-guide.ja.md)

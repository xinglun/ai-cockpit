---
author: AI Cockpit maintainers
workItemId: WI-944-release-v0-2-101
title: Release v0.2.101 with verification snapshot lifecycle repair
description: Prepare the reviewed candidate, then publish only after hosted and public-artifact evidence is complete.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-944-release-v0-2-101
---

[简体中文](WI-944-release-v0-2-101.zh-CN.md) · [日本語](WI-944-release-v0-2-101.ja.md)

# WI-944 — Release v0.2.101

This Work Item prepares the Runtime release containing the verification snapshot lifecycle repair. It is not a publication claim: tag, public Release, downloaded-artifact acceptance, and Sentinel replay remain pending until their evidence exists.

## Current scope

- Keep the workspace and current generated/reference projections at v0.2.101.
- Preserve recovery lineage when a successor binds an exact append-only recovery receipt.
- Require hosted CI, reviewed merge, immutable release assets, isolated adopter acceptance, and exact cleanup before terminal lifecycle steps.


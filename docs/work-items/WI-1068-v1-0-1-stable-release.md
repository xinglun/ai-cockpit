---
author: AI Cockpit maintainers
workItemId: WI-1068-v1-0-1-stable-release
title: stable v1.0.1 release
description: Deliver stable v1.0.1 from the existing reviewed line with public artifact and installation acceptance.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1068-v1-0-1-stable-release
---

[简体中文](WI-1068-v1-0-1-stable-release.zh-CN.md) · [日本語](WI-1068-v1-0-1-stable-release.ja.md)

# WI-1068 — stable v1.0.1 release

This Work Item delivers the existing v1.0.1 line as a true stable release, verifies the official artifacts, and updates the local CLI/MCP installation.

## Boundaries

- Product/source changes remain within the 16 declared product paths. The three language pages and parity rows are required Work Item governance projections.
- The real v1.0.0 stable release is the N-1 predecessor, and full N-1 acceptance runs on x86_64 Linux only.
- v1.0.1-rc.2 remains a separate immutable prerelease. The new stable tag is created only after merge and fresh Runtime publication admission. No existing tag, release, or asset may be replaced.
- The existing release workflow must pass its four-target candidate smokes and staged N-1 check before making the Release public. Public download/install acceptance and the macOS installed CLI/MCP checks follow publication.

The source candidate is under review in PR #1022. CI, publication, public acceptance, local installation, and cleanup remain pending until current evidence proves each stage.

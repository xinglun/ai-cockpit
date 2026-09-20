---
author: AI Cockpit maintainers
workItemId: release-v0-2-103
title: Release v0.2.103 after verification-receipt and distribution-policy repairs
description: Publish only after reviewed source, immutable public artifacts, adopter acceptance, and exact resource cleanup are evidenced.
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: release-v0-2-103
---

[简体中文](release-v0-2-103.zh-CN.md) · [日本語](release-v0-2-103.ja.md)

# Release v0.2.103

This Work Item prepares the Runtime release containing bounded verification-receipt handling and the Apple-Silicon-only Homebrew distribution policy. Publication is not claimed until the reviewed merge, immutable tag, public Release, downloaded-artifact checks, fresh install, upgrade, and provider cleanup evidence are complete.

## Boundaries

- The release includes the Runtime-side repair required before Sentinel can replay acceptance for Issue #902.
- Intel Homebrew is no longer a supported distribution or acceptance target; the standalone x86_64 macOS archive remains supported.
- It does not modify Sentinel or another adopter repository.
- It does not claim a measured performance improvement or third-party chat-display confirmation without direct evidence.

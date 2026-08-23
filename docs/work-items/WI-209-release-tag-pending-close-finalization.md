---
author: AI Cockpit maintainers
title: "WI-209 — release-tag finalization with truthful base"
description: "Bind the release successor to the synchronized default-branch base before verification and closure."
audience:
  - maintainer
  - adopter
workItemId: WI-209-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-209-release-tag-pending-close-finalization
---

# WI-209 — release-tag finalization with truthful base

WI-209 corrects the immutable WI-208 attempt by binding `baseRevision` to the
synchronized `origin/main` merge base (`56b5e8d0584743d4442d50156adf25a6e933eaf3`).
It records the exact PR #158 resource context through `finalize-plan` before
verification, then keeps the post-merge Runtime finalization and structured
human close pending until the published Runtime is available.

Entry points:

- [简体中文](WI-209-release-tag-pending-close-finalization.zh-CN.md)
- [日本語](WI-209-release-tag-pending-close-finalization.ja.md)

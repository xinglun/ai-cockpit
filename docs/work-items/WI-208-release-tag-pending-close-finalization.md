---
author: AI Cockpit maintainers
title: "WI-208 — release-tag finalization ordering"
description: "Bind PR resources before verification and preserve the post-merge Runtime closure boundary."
audience:
  - maintainer
  - adopter
workItemId: WI-208-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-208-release-tag-pending-close-finalization
---

# WI-208 — release-tag finalization ordering

WI-208 is the clean successor for the release-tag governance fix. It binds PR
#158, its branch, worktree, provider, and default branch through `finalize-plan`
before collecting verification evidence. The Work Item remains awaiting the
published Runtime's post-merge `finalize`, `finalize-verify`, and structured
human `close`; this boundary is not waived by the release-tag source gate.

Entry points:

- [简体中文](WI-208-release-tag-pending-close-finalization.zh-CN.md)
- [日本語](WI-208-release-tag-pending-close-finalization.ja.md)

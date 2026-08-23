---
author: AI Cockpit maintainers
title: "WI-207 — release-tag finalization ordering recovery"
description: "Preserve the successor that recorded verification before the Runtime-required finalize-plan boundary."
audience:
  - maintainer
  - adopter
workItemId: WI-207-release-tag-pending-close-finalization
status: recovered
authority: canonical
lastVerifiedBy: WI-207-release-tag-pending-close-finalization
---

# WI-207 — release-tag finalization ordering recovery

WI-207 is retained as immutable recovery history. It verified and archived a
successor before `finalize-plan`; the installed Runtime correctly rejected a
later finalization attempt because the required order is `finalize-plan` before
verification evidence. WI-208 continues with the exact PR context bound first.

Entry points:

- [简体中文](WI-207-release-tag-pending-close-finalization.zh-CN.md)
- [日本語](WI-207-release-tag-pending-close-finalization.ja.md)

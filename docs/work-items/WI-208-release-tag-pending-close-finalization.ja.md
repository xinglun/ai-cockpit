---
author: AI Cockpit maintainers
title: "WI-208 — release tag finalization order"
description: "verification 前に PR resource を bind し、公開後の Runtime close boundary を保持する。"
audience:
  - maintainer
  - adopter
workItemId: WI-208-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-208-release-tag-pending-close-finalization
---

# WI-208 — release tag finalization order

WI-208 は release-tag governance fix の clean successor です。verification evidence を
収集する前に `finalize-plan` で PR #158、branch、worktree、provider、default branch を
bind します。Work Item は公開された Runtime による merge 後の `finalize`、
`finalize-verify`、structured human `close` を待つ状態であり、Release tag source gate
がこの境界を免除することはありません。

文書入口：[English](WI-208-release-tag-pending-close-finalization.md) · [简体中文](WI-208-release-tag-pending-close-finalization.zh-CN.md)

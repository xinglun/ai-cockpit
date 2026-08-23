---
author: AI Cockpit maintainers
title: "WI-209——Release tag finalization 与真实 base"
description: "在 verification 和关闭之前，把 release successor 绑定到同步的默认分支 base。"
audience:
  - maintainer
  - adopter
workItemId: WI-209-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-209-release-tag-pending-close-finalization
---

# WI-209——Release tag finalization 与真实 base

WI-209 修正不可变的 WI-208 尝试，把 `baseRevision` 绑定到同步的
`origin/main` merge base（`56b5e8d0584743d4442d50156adf25a6e933eaf3`）。它在
verification 之前通过 `finalize-plan` 记录 PR #158 的准确 resource context，
并等待发布 Runtime 完成 merge 后 finalization 与结构化 human close。

文档入口：[English](WI-209-release-tag-pending-close-finalization.md) · [日本語](WI-209-release-tag-pending-close-finalization.ja.md)

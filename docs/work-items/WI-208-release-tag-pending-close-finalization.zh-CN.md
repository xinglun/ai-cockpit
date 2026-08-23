---
author: AI Cockpit maintainers
title: "WI-208——Release tag finalization 顺序"
description: "在 verification 前绑定 PR 资源，并保留发布后的 Runtime 关闭边界。"
audience:
  - maintainer
  - adopter
workItemId: WI-208-release-tag-pending-close-finalization
status: implemented
authority: canonical
lastVerifiedBy: WI-208-release-tag-pending-close-finalization
---

# WI-208——Release tag finalization 顺序

WI-208 是 release-tag 治理修复的 clean successor。它在收集 verification evidence
之前通过 `finalize-plan` 绑定 PR #158、branch、worktree、provider 和默认分支。
Work Item 仍等待发布 Runtime 完成 merge 后的 `finalize`、`finalize-verify` 与结构化
human `close`；Release tag source gate 不会豁免这个边界。

文档入口：[English](WI-208-release-tag-pending-close-finalization.md) · [日本語](WI-208-release-tag-pending-close-finalization.ja.md)

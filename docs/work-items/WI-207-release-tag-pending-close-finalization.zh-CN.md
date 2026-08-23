---
author: AI Cockpit maintainers
title: "WI-207——Release tag finalization 顺序恢复"
description: "保留在 Runtime 要求的 finalize-plan 边界之前记录 verification 的 successor。"
audience:
  - maintainer
  - adopter
workItemId: WI-207-release-tag-pending-close-finalization
status: recovered
authority: canonical
lastVerifiedBy: WI-207-release-tag-pending-close-finalization
---

# WI-207——Release tag finalization 顺序恢复

WI-207 作为不可变 recovery history 保留。它在 `finalize-plan` 之前完成了
verification 和 archive；已安装 Runtime 正确拒绝后续 finalization，因为要求的顺序是
先 `finalize-plan`，再记录 verification evidence。WI-208 会先绑定准确的 PR 上下文继续。

文档入口：[English](WI-207-release-tag-pending-close-finalization.md) · [日本語](WI-207-release-tag-pending-close-finalization.ja.md)

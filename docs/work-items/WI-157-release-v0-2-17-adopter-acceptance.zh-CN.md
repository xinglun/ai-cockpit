---
author: AI Cockpit maintainers
title: "WI-157——v0.2.17 发布与 adopter 验收"
description: "发布不可变 Runtime，并证明它可以治理全新的 adopter repository。"
audience:
  - adopter
  - contributor
  - maintainer
status: in_progress
authority: canonical
lastVerifiedBy: WI-157-release-v0-2-17-adopter-acceptance
workItemId: WI-157-release-v0-2-17-adopter-acceptance
---

# WI-157——v0.2.17 发布与 adopter 验收

本 Work Item 只有在 source、package 和文档 identity 一致后才发布新的
Runtime。发布后的验收只使用不可变公开 archive，不使用 workspace 构建或
fallback binary，并记录 Runtime digest、adopter repository identity、隔离
manifest、evidence reuse、scaffold 的 `not_ready` 状态以及完整 Work Item
lifecycle receipt。

验收 receipt 属于发布后 evidence。验收失败不能回写已经发布的 Release truth。

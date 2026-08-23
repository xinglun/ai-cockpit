---
author: AI Cockpit maintainers
title: "WI-157——v0.2.17 发布与 adopter 验收"
description: "发布不可变 Runtime，并证明它可以治理全新的 adopter repository。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
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

Release：https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.17

Workflow：https://github.com/xinglun/ai-cockpit/actions/runs/32606940727

本地公开 artifact evidence：`.ai/evidence/external/v0.2.17/adopter/` 与
`.ai/evidence/external/v0.2.17/upgrade/`。已安装的公开 binary 为 Runtime
`0.2.17`，digest 为
`sha256:4157cc04a23a24e6ac618e7079c123210920fba2e7fc5335c9f6a734c74721e3`。
发布前的 v0.2.16 evidence bytes 保存在
`.ai/evidence/external/v0.2.16/WI-157-release-v0-2-17-adopter-acceptance/`，
不会被重新当作当前 verification evidence 使用。

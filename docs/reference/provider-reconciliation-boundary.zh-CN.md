---
author: AI Cockpit maintainers
title: "Provider 对账边界"
description: "历史 provider 清单是证据上下文，不是当前 provider 事实。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Provider 对账边界

[English](provider-reconciliation-boundary.md) · [日本語](provider-reconciliation-boundary.ja.md)

`open-pr-issue-reconciliation-662.*` 和 `pre-release-documentation-alignment.json`
等参考文件，是源仓库历史版本的 provider/审查评估材料。它们记录过去某个版本
观察到的状态，不能证明当前仓库、当前 GitHub PR、发布或企业批准。

AI Cockpit 保持 provider 责任边界：

- Runtime 可以要求、绑定、展示和归档委托证据；
- GitHub/Hosted CI、审查者、分支保护、发布和企业留存由外部系统负责；
- 过期或缺失的对账是 unknown，不能授权 merge、release 或 close；
- 新 provider 观察必须针对当前仓库和 Work Item 身份重新采集，并带有自己的
  digest、时间和来源。

因此，逐文件台账将源 JSON/Markdown 记录标为 `reference-only`。它们不会复制到
`.ai/`，不会并入当前 status，也不会覆盖仓库本地 Contract 或 Runtime 证据。
目标工程边界请参见[发布分发](../release/distribution.zh-CN.md)和[参考源对齐](reference-parity.zh-CN.md)。

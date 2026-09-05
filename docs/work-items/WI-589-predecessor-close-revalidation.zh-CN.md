---
author: AI Cockpit maintainers
title: "WI-589——Contract 修订重验证后的前置 Work Item 关闭"
description: "在后继 Work Item 重验证修订后的 Contract 后，为旧 Provider 收据提供绑定关闭的历史投影。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-589-predecessor-close-revalidation
lastVerifiedBy: WI-589-predecessor-close-revalidation
terminalArchive: .ai/work-items/archive/WI-589-predecessor-close-revalidation.contract.json
terminalVerification: .ai/evidence/WI-589-predecessor-close-revalidation.verification.json
terminalFinalization: .ai/decisions/WI-589-predecessor-close-revalidation.finalize.046eea80433b45884c522474d2bca7da061b2056187418e638d962d86699db3d.json
terminalDecision: .ai/decisions/WI-589-predecessor-close-revalidation.close.json
---

[English](WI-589-predecessor-close-revalidation.md) · [日本語](WI-589-predecessor-close-revalidation.ja.md)

# WI-589——Contract 修订重验证后的前置 Work Item 关闭

## 目标

当经过审查的 Contract 修订已由一个终态后继 Work Item 重新验证后，允许归档的前置 Work Item 诚实关闭。旧 Provider finalization 收据仍是历史证据：其字节、路径、摘要和序号都保持不变，绝不会被重新标记为 `direct_merge_no_pr`。

## 边界

这是严格、追加式的兼容路径。后继项必须完成当前 Runtime 的验证、Provider finalization 和显式人工关闭；只有此后，前置项才可将精确的旧 finalization head 以 `historical_low` 重验证绑定到 close。缺失、格式错误、外部身份、过期或矛盾的 lineage 仍然 fail-closed。direct-merge schema、adopter 脚本、对象工程和参考源实现不属于本 Work Item。

## 验收

1. 旧 Runtime 产生的 PR 收据只有在 Contract 修订后继项终态且与仓库绑定时，才能投影为历史证据。
2. 前置 close 记录精确的 finalization 路径、摘要和序号，并保留原始收据字节。
3. close 记录区分当前后继重验证与历史 Provider 证据，不伪造 PR，也不伪造 direct-merge 分类。
4. 未完成或被篡改的后继项、归档、Contract、证据或收据绑定必须 fail-closed，且不得生成半成品 close 记录。

## 验证

运行 recovery 专项回归测试和完整的 locked workspace 测试。三语命令参考会说明支持的恢复路径及其 fail-closed 边界。

---
author: AI Cockpit maintainers
title: "WI-435——本地参考源逐文件台账重新基线"
workItemId: WI-435-reference-inventory-rebaseline-local
description: "将逐文件参考台账绑定到维护中的本地语义参考源，不静默提升变化源文件的历史决定。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-435-reference-inventory-rebaseline-local
---

# WI-435——本地参考源逐文件台账重新基线

本 Work Item 将逐文件比较台账显式绑定到由维护者提供、通过
`AI_COCKPIT_REFERENCE_ROOT` 选择的本地 checkout。源固定在提交
`fde3380f81fea5fd2e288f7a8849f737dc074060`；不需要公开参考仓库。这是台账与文档变更，
不是语义比较批次，也不复制源内容。

[English](WI-435-reference-inventory-rebaseline-local.md) · [日本語](WI-435-reference-inventory-rebaseline-local.ja.md)

## 范围与安全边界

- 记录当前 4,450 条 tracked path、160 条变化路径，以及相对上一台账退休的 669 条路径。
- 保留每条旧决定作为历史；变化的非历史记录继续标记为 `deferred-next-batch`，直到后续逐文件复核。
- 保留旧源提交和台账摘要，使机器台账、lock、测试和三语文档一致。
- 不复制参考源文件，不改变 Rust Runtime 行为，不修改 CI 策略，也不从源更新推断治理决定。

当前台账包含 3,681 条 generated-history、223 条
implemented-different-by-design、1 条 implemented-equivalent、4 条
not-applicable、62 条 reference-only 和 479 条 deferred-next-batch。退休路径是历史元数据，
不是当前 parity 声明。

## 验证边界

只有在本地源策略、旧台账回归、当前台账回归、文档检查、parity 检查和 workspace 测试全部通过时，
才能接受重新基线。变化或移除的源路径必须在台账中可见；缺少 checkout、移动中的提交或公开网络
fallback 都视为失败。

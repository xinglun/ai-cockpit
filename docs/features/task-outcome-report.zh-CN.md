---
author: AI Cockpit maintainers
title: "Task Outcome 报告"
description: "说明 Work Item 完成、发现、停止和留给人审阅的 evidence-bound 报告。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-457
capabilityClaims:
  - task_outcome_report
---

# Task Outcome 报告

Rust Runtime 保留 `OutcomeV2` 作为稳定机器对象，并为新生成的 Outcome
增加可选的 `taskOutcomeReport` 投影。该投影严格绑定 repository，并以增量
方式加入；没有该投影的历史 OutcomeV2 bytes 仍可读取，不会被重写。

报告明确包含：结果摘要、任务概览、交付变更、发现的问题、风险、警告、限制、
干预、强制停止、解决、复发预防、避免的影响、剩余风险、人工决定、实施方式和
证据。空 section 显示为 `None`，不表示检查已完成或收益存在。

每个非空 claim 都必须带 repository-local `evidenceRefs`，或显式标为
`inference`。Contract 的 intent、scope、验收标准和 authority 仍是人类编写的
源文本。Runtime 不会推断用户收益、合并、发布、provider 批准、企业保证或安全
结论。

## 生命周期产物

`finish` 后，active Work Item 会包含 `<id>.outcome.json` 和追加写入的
`<id>.events.jsonl`。事件记录生成的完成、finding、risk、warning、stop、resolution、
recurrence prevention 和有证据的 check-pass-after-fix。`archive` 会
逐字节移动事件流，并在 archive manifest 中绑定 digest。`close` 会把已校验的
报告作为 `finalReport` 写入 repository-bound close receipt，并写入
`finalReportDigest`。

失败或中断的生命周期尝试还可能留下 Runtime 生成的投影，例如
`<id>.outcome.finish-blocked.json` 或 `<id>.events.finish-recovery.jsonl`。
这些是审计历史，不是活跃 Contract；因此 `status` 会通过
`activeArtifacts` 和 `orphanedActiveArtifacts` 单独报告，而
`activeWorkItems` 仍只按 Contract 统计。正常 `archive` 会将识别出的变体与
规范文件一并移动，并在 `historicalArtifacts` 中记录每个摘要。对于旧 Runtime
已经归档的 Work Item，使用
`ai-cockpit work-item reconcile-artifacts --repo <repository> --id <id>`。
该命令要求并验证现有 archive manifest，只移动与身份绑定的普通文件，并写入
追加式 reconciliation receipt；不会删除或改写历史字节。

`archive` 与 `close` 是两个不同的边界。归档后的 Work Item 仍需显式的人类决定
才能关闭；孤立投影不代表 Work Item 仍在进行，但在归档或 reconciliation 前会阻塞
仓库 ready 状态。

事件流会拒绝 malformed JSON、未知字段、foreign repository/Work Item identity、
不安全 evidence 路径、疑似 secret 内容、重复 ID，以及引用尚未出现事件的关系。
修正必须新增事件，不能静默改写历史行。Finding/risk 事件带确定性的
`findingFingerprint`；重复值只有在显式 correction/supersession 绑定时才可接受。
Rust 是参考源事件策略的语义 parity，不是 source JSON 的字节兼容，也不会导入参考源 Python generator。

## 面向人的交接

CLI `ai-cockpit work-item outcome --repo <repository> --id <id>` 与 MCP
`work_item_outcome` 使用同一份已校验报告和 renderer。交接显示状态标记、完成内容、
问题、停止、解决、风险、未知项、决定、验证、影响和下一步。Runtime 生成的标签会
本地化；Contract 源文本保持原语言。

报告本身不授权合并、发布、provider 批准或组织决策。事件驱动的 paused/blocked/
stale/cancelled/rollback 状态重建仍属于后续 recovery 能力。

[Human Benefit 报告](human-benefit-report.zh-CN.md) | [Outcome 参考](../reference/outcome-report.zh-CN.md) |
[功能](README.zh-CN.md) | [English](task-outcome-report.md) | [日本語](task-outcome-report.ja.md)

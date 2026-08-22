---
author: AI Cockpit maintainers
title: "WI-122——Scenario、Acceptance 与最终维度控制"
description: "为 Contract/Summary 治理投影增加有界验证和显式记录能力。"
audience:
  - adopter
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: work-item-acceptance
capabilityClaims:
  - scenario_coverage
  - acceptance_evidence
  - final_dimensions
---

# WI-122——Scenario、Acceptance 与最终维度控制

WI-122 增加只读 Contract/Summary 验证器和有界 `controls` 写入口。高风险
scenario coverage fail-closed；没有编号的旧 Acceptance 保持兼容；有编号的
Acceptance 使用稳定 ID 和逐项 evidence。Intent alignment 在未知时保持显式。

最终验收 receipt 使用参考源完整的 20 个维度。Runtime 验证 receipt 的结构、
身份、决定和 GO 前置条件，但不会合成 provider、enterprise 或 adopter evidence。
可选的 `fourPillarProjection` 是明确命名的展示投影，不是 `4D` 字段。

功能通过 `work-item validate`、`work-item controls` 和 repository-bound MCP
工具 `work_item_validate` 提供。所有命令都要求显式 repository 绑定。

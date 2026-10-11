---
author: AI Cockpit maintainers
title: "WI-1076 — 治理门禁中的已恢复前驱投影"
description: "让治理完整性投影与状态一致性门禁对严格验证的已替代前驱作出一致判定。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1076-governance-recovered-predecessor-projection
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1076-governance-recovered-predecessor-projection
contractDigest: sha256:988b6c0d670c237df062bd3b90867b23e39c512eb6774400043a1dac8042f893
---

[English](WI-1076-governance-recovered-predecessor-projection.md) · [日本語](WI-1076-governance-recovered-predecessor-projection.ja.md)

# WI-1076 — 治理门禁中的已恢复前驱投影

本页投影当前 [WI-1076 Contract](../../.ai/work-items/active/WI-1076-governance-recovered-predecessor-projection.contract.json)，绑定摘要 `sha256:988b6c0d670c237df062bd3b90867b23e39c512eb6774400043a1dac8042f893`。准入、验证和生命周期状态以 Runtime 记录为准。

## 预期收益

对于同一条有效的已替代前驱 lineage，两套治理门禁应报告相同的前驱状态。只有 recovery、close、repository 和 successor 绑定均通过摘要验证时，前驱才能标记为 `Recovered`。尚未完成的后继项必须继续显示为 `awaiting_successor_close`，不得投影为终态。

## 范围与边界

实现文件仅限 `tests/ci/governance_integrity_gate.py` 和 `tests/ci/governance_integrity_gate_test.sh`。三语页面和 parity 行属于必需的 Work Item 治理投影。另有三份范围内页面保留 WI-1068 稳定版发布历史，来源为经审阅的 WI-1070 文档提交 `b2402611267b5a8c65c15e834e462ea74d0b42ad`；WI-1070 治理记录和发布验收状态仍不属于本 Work Item。状态一致性检查器、Runtime、产品代码、发布工作流、tag、发布和 provider 资源均不在范围内。现有 acceptance、archive 和 close 要求继续作为硬门槛。

## 验收与验证

验收需覆盖有效的已替代前驱及其尚未完成且身份绑定的后继；覆盖无效 recovery、close、repository 或 lineage 绑定的拒绝情形；并保留后继项未完成状态。只有通过验证、已归档且确认关闭的后继项才能使前驱转为已实现。

Contract 声明运行 `bash tests/ci/governance_integrity_gate_test.sh`、`bash tests/docs/work_item_status_consistency_test.sh` 和 `bash tests/docs/parity_status_check.sh`。检查结果以绑定当前 Work Item 的 Runtime 正式验证回执为准，本页不替代该证据。

---
author: AI Cockpit maintainers
workItemId: WI-865-four-direction-convergence
title: "四方向收敛验收"
description: "性能、Outcome、CHI 与架构收敛的证据化验收报告。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-865-four-direction-convergence
terminalArchive: .ai/work-items/archive/WI-865-four-direction-convergence.contract.json
terminalVerification: .ai/evidence/WI-865-four-direction-convergence.verification.json
terminalDecision: .ai/decisions/WI-865-four-direction-convergence.close.json
---

# WI-865——四方向收敛验收

## 范围与决定边界

本报告记录 WI-865 Contract 的四个有界工作包，不授予合并或发布授权。审阅合并后发布仍由单独且明确授权的 Work Item 负责。

## 验收证据

- **性能（P）：** 发布级统计和 P0 比较器拒绝 99 个有效 warm 样本、接受 100 个，保留原始样本，并把不可取得的计数写成原因。采集器覆盖 Runtime inspect/status/Outcome/验证规划和 diagnose；配对 JSON 在采集后链接。
- **Outcome（O）：** lifecycle、CLI、MCP、summary 和 full 都消费 Runtime 绑定的观察组装。发布事实是可选的，必须有明确绑定证据；changed paths 仅保留在审计细节。
- **CHI（C）：** 三语首次 Work Item 路径以 `start --prepare` 开始，区分可评审、可合并、已收尾，并把 provider/release 细节放入详细工作流参考。
- **架构（A）：** 观察、生命周期、验证、投影和适配器所有权写入职责图，并由 Outcome/lifecycle 定向测试保护。

## 当前状态

下表只在有 Runtime receipt、托管 PR 状态和配对 benchmark JSON 时填写。不可用或未知字段保持原样，不从绿灯推断收益或授权。

| 方向 | 基线 → 候选 | 直接证据 | 结论 |
|---|---|---|---|
| Runtime 与周期成本 | 等待配对采集 | `.ai/evidence/WI-865-four-direction-convergence/` | 待验收 |
| Outcome 一致性 | lifecycle 与投影测试 | `.ai/evidence/WI-865-four-direction-convergence.verification.json` | 待验收 |
| CHI/默认路径 | 三语语义和文档门禁 | `tests/docs/getting_started_semantic.sh` | 待验收 |
| 架构 | 职责图与纯 renderer 测试 | `docs/reference/architecture-responsibility-map-2026-09.md` | 待验收 |

## 剩余风险

Resident MCP 和 provider 端发布是本 Work Item 的外部边界。后续 release Outcome 只有在取得各自不可变记录及证据引用时，才会填入版本、发布链接、安装/升级验收和清理状态。

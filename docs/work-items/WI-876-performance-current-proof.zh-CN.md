---
author: AI Cockpit maintainers
workItemId: WI-876-performance-current-proof
title: 当前版本性能证明
description: 为四方向收敛验收建立配对的 Runtime 与开发周期测量。
audience: [adopter, contributor, maintainer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-876-performance-current-proof
terminalArchive: .ai/work-items/archive/WI-876-performance-current-proof.contract.json
terminalVerification: .ai/evidence/WI-876-performance-current-proof.verification.json
terminalDecision: .ai/decisions/WI-876-performance-current-proof.close.json
---

# WI-876——当前版本性能证明

本 Work Item 在同一机器、工具链和场景集合上建立可比较的 Runtime 延迟与开发周期成本证据，不发布版本，也不改变 Runtime 行为。

## 验收边界

- 发布级分位数比较至少需要 100 个有效 warm 样本；99 个只能诊断，必须被门禁拒绝。
- 配对测量覆盖 inspect、status、Outcome、验证规划与复用、仓库规模/历史、单文件/多文件变更，以及证据失效并明确写出不可取得原因。
- 每个外部样本绑定适用的读取、哈希、解析、Git、子进程、执行和复用计数；如无法取得则说明原因，并报告区间重叠和诊断开关开销。
- Contract→可评审、验证→finish、合并后清理分别报告原始样本、p50/p95、Agent 操作次数和前置拒绝次数。

Runtime 或宿主无法暴露的内部指标保留为未知。本 Work Item 不包含发布。

---
author: AI Cockpit maintainers
title: "协作场景矩阵"
description: "由状态与转换生成的协作语言场景矩阵，明确区分实际观测案例、既有文档案例与人工构造案例。"
audience: [adopter, contributor, maintainer, reviewer]
status: current
authority: canonical
lastVerifiedBy: WI-781-trust-diagnostics
---

# 协作场景矩阵

本页是 [`collaboration-scenario-matrix.json`](collaboration-scenario-matrix.json)
的可读配套文档；该 JSON 才是供自动化检查使用的结构化事实来源。本页扩展当前的
[协作语言契约](collaboration-language-contract.zh-CN.md)，场景依据仓库支持的
生命周期、证据、授权、组合执行与操作语义编写。

## 如何阅读本矩阵

每条场景都标明：

- **来源类型(sourceType)**——`observed`(2026-09-08 在本仓库交付 WI-679/
  WI-680 期间实际执行，附确切命令/输出)、`documented`(转述既有权威文档，
  未新增执行)、或 `designed`(合成场景，仅在某个必需类别缺乏可用真实记录
  时构造，从不宣称已被证明的真实使用效果)。
- **类别(category)**——生命周期转换、证据、授权、验证、合并/关闭、历史查询、
  Agent/会话交接、多语言/多入口一致性、组合执行/重试、进程监督或旧版组合记录。
- **预期结果**与它所验证的**语义不变量**(引用协作语言契约中的十条不变量)。

按照本专项自身的范围要求，本矩阵优先覆盖关键边界与容易混淆的组合，而不是
穷举每个状态与每个操作的完整笛卡尔积。

## 覆盖情况汇总

| 类别 | 场景 | 实际观测 | 既有文档 | 人工构造 |
| --- | --- | --- | --- | --- |
| 生命周期转换 | SCN-001..005 | 4 | 0 | 0(SCN-005 是操作层面的隐患，不是治理拒绝) |
| 证据 | SCN-006..008 | 1 | 2 | 0 |
| 授权 | SCN-009..012 | 1 | 3 | 0 |
| 验证 | SCN-013..015、SCN-025 | 3 | 1 | 0 |
| 合并/关闭 | SCN-016..019 | 3 | 1 | 0 |
| 历史查询 | SCN-020 | 0 | 1 | 0 |
| Agent/会话交接 | SCN-021..022、SCN-026 | 2 | 1 | 0 |
| 多语言/多入口 | SCN-023..024 | 0 | 2 | 0 |
| 收尾状态观测 | SCN-027..033 | 6 | 0 | 0 |
| 组合执行 | SCN-034 | 0 | 1 | 0 |
| 组合重试 | SCN-035..036 | 0 | 2 | 0 |
| 进程监督 | SCN-037 | 0 | 1 | 0 |
| 旧版组合记录 | SCN-038 | 0 | 1 | 0 |
| 验证重试边界 | SCN-039 | 0 | 1 | 0 |

SCN-033 的状态为 unavailable，单独列出，不计入上表的实际观测、既有文档或人工构造数量。

SCN-034 至 SCN-039 是根据 Contract A12-A14 和当前实现整理的 `documented`
场景，不表示这些结果已在本次云端执行中观测到。本次没有新增 `designed` 场景。

## 完整矩阵

完整的、结构化的场景表(SCN-001 至 SCN-039;编号稳定，未来 Work Item 只能
扩展、不能重新编号)见 `collaboration-scenario-matrix.json`。以下为代表性
样例:

| 编号 | 类别 | 标题 | 来源 | 结果 |
| --- | --- | --- | --- | --- |
| SCN-003 | 生命周期转换 | 前序 WI 未获得有效关闭决定时，新 WI 起票被拒绝 | 实际观测 | 拒绝 |
| SCN-006 | 证据 | 验证后 Contract 变更导致证据失效，`finish` 被拒绝 | 实际观测 | 拒绝 |
| SCN-013 | 验证 | 缺少 `acceptanceEvidence`/`intentAlignment` 时 `finish` 被阻断 | 实际观测 | 拒绝 |
| SCN-014 | 验证 | `finish` 成功(绿色)但明确不授权合并 | 实际观测 | 通过 |
| SCN-017 | 合并/关闭 | 合并确认被平台(而非 Runtime)权限边界拒绝 | 实际观测 | 拒绝 |
| SCN-019 | 合并/关闭 | 合并前发现的文档缺口由后续 Work Item 修复，而非改写历史 | 实际观测 | 推迟给后续WI |
| SCN-022 | Agent/会话交接 | 一个 Agent 未完成的前序关闭，结构性地阻止另一个 Agent 的新起票 | 实际观测 | 待前序关闭前拒绝 |
| SCN-025 | 验证 | 展示的选项、选择的测试数据决定、中断与恢复后的 Runtime 转换保持一致 | 实际观测 | 可安全重试的一致状态 |
| SCN-026 | Agent/会话交接 | 新子进程仅从 Runtime 记录重建交接，不依赖会话历史 | 实际观测 | 带明确阻断的状态可完全恢复 |
| SCN-034 | 组合执行 | 执行通过但清理延迟时仍为 unknown，且不可复用 | 既有文档 | 非终态 unknown |
| SCN-036 | 组合重试 | 仅绑定的 Linux 系统 no-op 可进行 deferred-cleanup fresh retry | 既有文档 | 新 attempt，保留旧目录 |
| SCN-037 | 进程监督 | Unix/Windows process-group backend 不声称 Linux ECHILD 证明 | 既有文档 | 按 backend 区分 |
| SCN-038 | 旧版组合记录 | legacy v1/v2 布尔值不能建立可复用的 v3 证据 | 既有文档 | 仅兼容显示 |
| SCN-039 | 验证 | Cargo 首次运行不代表支持 deferred-cleanup 重试 | 既有文档 | spawn 前阻断重试 |

## 已知局限

本矩阵仍是人工整理的语义索引，而不是完整的自动化测试套件。其 JSON 来源现已
包含可执行检查注册表：每个绑定场景都指明输入事实、预期语义和测试入口。WI-781
增加了人类报告语义对齐、finalization 动作分类、有界 Outcome 组装重试和分阶段
Runtime 诊断；不支持的进程计数明确保持 unavailable。任何检查都不宣称覆盖每个
不变量或状态组合的完整笛卡尔积；未来 Work Item 可以继续增加有界检查，但不得
重新编号已有场景。

新增组合场景是文档边界，不是 hosted acceptance receipt。Cargo 命令可以作为
获准的首次执行运行；当前 deferred-cleanup retry 政策只接受一个受保护的系统
no-op `true`，不表示支持 Cargo 或任意测试重试。投影也不能替代 Work Item 的
`finish` gate。

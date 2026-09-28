---
author: AI Cockpit maintainers
workItemId: WI-1036-task8-ci-recovery
title: Task 8 CI 收尾
description: 仅解决 PR #997 中观察到的三类 CI 失败，保留必需门禁，合并并清理 Task 8，在发布前停止。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-1036-task8-ci-recovery
terminalArchive: .ai/work-items/archive/WI-1036-task8-ci-recovery.contract.json
terminalVerification: .ai/evidence/WI-1036-task8-ci-recovery.verification.json
terminalDecision: .ai/decisions/WI-1036-task8-ci-recovery.close.json
---

[English](WI-1036-task8-ci-recovery.md) · [日本語](WI-1036-task8-ci-recovery.ja.md)

# WI-1036 — Task 8 CI 收尾计划

这是 Task 8 最后且范围受限的 CI 收尾，仅处理 [PR #997](https://github.com/xinglun/ai-cockpit/pull/997) 的 [CI run 36363705256](https://github.com/xinglun/ai-cockpit/actions/runs/36363705256) 所记录的失败：`cockpit-verification` 的 7 个 composition 测试无法检查进程文件描述符；前驱 WI 在合并后投影前仍显示条件状态，导致 `docs_closed_work_item_promotion` 提前拒绝；`ci_manifest_regression` 未能到达预期的缺少 receipt 诊断。

进程观察器的根因尚未证实。现有实现会在可能持有临时 worktree 的进程不可检查时 fail closed。修改行为前，先判断这是实际所有权歧义，还是无关进程或调度交互。

## 有界实施计划

1. 保留失败 CI run、日志和当前 PR head。仅在 Runtime 确认身份及新鲜度完全匹配时复用既有 receipt；除非当前 Runtime 明确要求，不重跑归档的 14 节点 hosted 验证。
2. 为已报告的 composition 失败补充聚焦的失败回归。查明进程所有权观察问题，再做最小安全修复；所有权未知不得变成清理或复用仍活动 worktree 的许可。
3. 添加按生命周期阶段区分的文档晋级测试。合并前 CI 不要求合并后的投影；同步主线后的晋级要求仍必须执行并受测试保护。
4. 隔离缺少 receipt 的负例，使无关的当前仓库文档状态无法掩盖预期诊断；保留畸形和错误 receipt 的检查。
5. 执行新鲜 Runtime preflight，然后在本 WI 中串行运行其准入的针对性检查。由 canonical PR CI 运行完整必需门禁；不跳过或削弱门禁。
6. 将修正 PR 集成到 Task 8 分支，确认 PR #997 精确 head 上的必需检查通过，再合并 PR #997 并完成精确的本地 WI 清理。
7. 关闭 Task 8 后直接进入 Task 9。发布、打 tag 或公开发布前停止。

## 可审查的变更切片

- composition 所有权／调度修正及聚焦 Rust 回归测试。
- 按生命周期阶段判断文档晋级资格，并回归证明合并后晋级仍为必需。
- 确定性的缺少 receipt manifest 回归测试。
- 本三语 Work Item 计划／状态投影。

本 WI 不包含新增协作能力、泛化指南重设计、无关 Task 8 修复或 Task 9 迁移。当前状态为进行中；本计划不声称任何修复或验证已通过。

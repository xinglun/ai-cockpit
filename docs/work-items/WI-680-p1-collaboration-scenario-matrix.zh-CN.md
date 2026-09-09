---
author: AI Cockpit maintainers
title: "WI-680——P1 协作场景矩阵"
description: "由状态与转换生成的协作场景矩阵，并在 reference-parity 台帳中以三语言登记 WI-679 与 WI-680。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-680-p1-collaboration-scenario-matrix
status: implemented
authority: authorized
lastVerifiedBy: WI-680-p1-collaboration-scenario-matrix
terminalArchive: .ai/work-items/archive/WI-680-p1-collaboration-scenario-matrix.contract.json
terminalVerification: .ai/evidence/WI-680-p1-collaboration-scenario-matrix.verification.json
terminalFinalization: .ai/decisions/WI-680-p1-collaboration-scenario-matrix.finalize.json
terminalDecision: .ai/decisions/WI-680-p1-collaboration-scenario-matrix.close.json
---

[English](WI-680-p1-collaboration-scenario-matrix.md) · [日本語](WI-680-p1-collaboration-scenario-matrix.ja.md)

# WI-680——P1 协作场景矩阵

## 意图

推进 AI Cockpit 协作语言专项的 P1 范围:根据仓库实际支持的生命周期、证据
状态、授权状态与操作类型,生成场景矩阵,优先覆盖关键边界与容易混淆的组合,
而非穷举。同时补齐 WI-679 因无法在正确的"归档前"提交顺序下完成、又不愿改写
已推送历史而遗留的 reference-parity 台帳登记。

## 边界

这是仅限文档的 Work Item。新增
`docs/reference/collaboration-scenario-matrix.md`(+ zh-CN/ja)、
`docs/reference/collaboration-scenario-matrix.json`(结构化事实来源)、
`docs/reference/README.md`(+ zh-CN/ja)中的一条索引条目、
`docs/work-items/WI-679-*` 与 `docs/work-items/WI-680-*` 记录页面,以及
`docs/reference/reference-parity.*` 中登记 WI-679 与 WI-680 的行。不改变任何
CLI、MCP、Contract schema、Outcome schema 或生命周期行为。将场景转化为针对
受控测试仓库的自动化可执行检查明确不在本次范围内,已在交付文档中列为后续
工作。

## 授权记录

2026-09-08,人类已明确授权通过 Work Item 流程继续推进协作语言专项,包括
修复交付过程中发现的文档登记缺口。确切范围与证据保存在 Contract 与 PR 中。

## 验收与生命周期

- 场景矩阵的结构化 JSON 覆盖所有必需类别,每条都标注 sourceType(observed/
  documented/designed),其中大多数引用本次交付中产生的真实命令/输出。
- WI-679 与 WI-680 均以"归档前"的 `进行中 → 验证关闭后已实现` 状态出现在
  三语言 reference-parity 台帳中,每行的首次出现都先于本 Work Item 自身的
  verification 证据提交。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`;
  保留 `user_visible_benefit_not_declared`。
- 在精确审查的 head 上,`bash tests/docs/documentation_acceptance.sh` 通过。

## 证据

- archive:`.ai/work-items/archive/WI-680-p1-collaboration-scenario-matrix.contract.json`
- verification:`.ai/evidence/WI-680-p1-collaboration-scenario-matrix.verification.json`
- finalization:`.ai/decisions/WI-680-p1-collaboration-scenario-matrix.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-680-p1-collaboration-scenario-matrix.close.json`(待合并后生成)

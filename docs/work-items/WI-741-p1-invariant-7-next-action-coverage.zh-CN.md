---
author: AI Cockpit maintainers
title: "WI-741 — P1 不变量7(下一步动作正确性)覆盖"
description: "以绑定协作场景矩阵一部分实际观测场景的有界测试,弥补 docs/reference/collaboration-invariant-coverage.md 指出的不变量7缺口。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-741-p1-invariant-7-next-action-coverage
status: in_progress
authority: authorized
lastVerifiedBy: WI-741-p1-invariant-7-next-action-coverage
---

[English](WI-741-p1-invariant-7-next-action-coverage.md) · [日本語](WI-741-p1-invariant-7-next-action-coverage.ja.md)

# WI-741 — P1 不变量7(下一步动作正确性)覆盖

## 意图

弥补 `docs/reference/collaboration-invariant-coverage.md` 指出的唯一剩余
缺口:不变量7"展示的下一步与当前 Runtime 状态/策略一致",此前没有任何
测试将渲染出的恢复/下一步文本与独立记录的期望值进行断言比对。对每一种
Runtime 状态建立通用判定器并不现实,因此本 Work Item 采取该文档已经建
议的有界做法:针对
`docs/reference/collaboration-scenario-matrix.json` 中
`sourceType: "observed"` 的一个子集场景(SCN-001、SCN-002、SCN-016),将
其精确的下一步文本与各场景记录的 `expected.keyMessage` 进行断言比对。基
于仓库所有者的明确委托,继续推进 AI Cockpit 协作语言专项。

## 边界

这是一个仅测试的 Work Item。它只新增一个文件:
`crates/cockpit-repository/tests/scenario_matrix_next_action.rs`。不修改
任何生产源代码、任何既有测试文件,也不修改协作不变量覆盖文档或场景矩阵
本身(将不变量7的行更新为反映本次新增覆盖是明确的、独立的后续工作,不
属于本次交付范围,因此本 Work Item 不能被解读为默默改写其前置 Work Item
的结论)。

## 受理与生命周期

- `crates/cockpit-repository/tests/scenario_matrix_next_action.rs` 断言,
  通过直接调用 `cockpit_repository` 库函数产生的精确(SCN-001、SCN-016)
  或子串(SCN-002)下一步文本,与
  `docs/reference/collaboration-scenario-matrix.json` 中各场景记录的
  `expected.keyMessage` 一致。
- 同一文件中的第四个守护测试确认 SCN-001、SCN-002、SCN-016 仍被声明为
  `sourceType: "observed"` 并继续覆盖不变量7,使本测试不会悄然偏离其应
  当守护的文档。
- `cargo test -p cockpit-repository --test scenario_matrix_next_action`、
  `cargo fmt --check`、`cargo clippy --tests -- -D warnings` 均通过。
- `start → preflight → checkpoint → verify → finish → archive → close` 是被
  治理的路径;`user_visible_benefit_not_declared` 保持明确。

## 证据

- archive: `.ai/work-items/archive/WI-741-p1-invariant-7-next-action-coverage.contract.json`
- verification: `.ai/evidence/WI-741-p1-invariant-7-next-action-coverage.verification.json`
- finalization: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.finalize.json`(合并后生成)
- close: `.ai/decisions/WI-741-p1-invariant-7-next-action-coverage.close.json`(合并后生成)

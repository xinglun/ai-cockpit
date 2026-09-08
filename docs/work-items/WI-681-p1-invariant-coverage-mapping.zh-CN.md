---
author: AI Cockpit maintainers
title: "WI-681——P1 协作不变量覆盖映射"
description: "将十条协作语言语义不变量映射到既有自动化测试覆盖，引用确切测试，并将剩余缺口列为有边界的后续 Work Item。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-681-p1-invariant-coverage-mapping
status: in_progress
authority: authorized
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

[English](WI-681-p1-invariant-coverage-mapping.md) · [日本語](WI-681-p1-invariant-coverage-mapping.ja.md)

# WI-681——P1 协作不变量覆盖映射

## 意图

推进 AI Cockpit 协作语言专项的 P1 范围:针对 WI-679 协作语言契约中陈述的
十条语义不变量,逐一回答本仓库自身测试套件中是否已经有自动化测试在强制
执行,并引用确切的测试。这是一次"优先复用"的调查:目的是避免重复建设测试
基础设施,并如实说明哪些不变量尚无自动化交叉检查,依据仓库所有者的明确
授权,通过 Work Item 流程继续推进本专项。

## 边界

这是仅限文档的 Work Item。新增
`docs/reference/collaboration-invariant-coverage.md`(+ zh-CN/ja)、
`docs/reference/README.md`(+ zh-CN/ja)中的一条索引条目、本记录页面,以及
自身在 reference-parity 中的登记。不修改任何测试文件、任何 CLI/MCP 行为,
也不修改任何 Contract/Outcome schema。编写所发现的四项缺失测试明确不在本
次范围内,已在交付文档中列为有边界的后续 Work Item,且每一项都引用一个
可扩展的既有测试模式,而不是发明新模式。

## 验收与生命周期

- 每一条"是"/"部分"的判定都引用了在交付时直接阅读当前测试源码所确认的
  测试文件与函数。
- 每一项缺口("未发现自动化测试")都被准确陈述,并给出具体的、基于复用的
  后续方案。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`;
  保留 `user_visible_benefit_not_declared`。
- 在精确审查的 head 上,`bash tests/docs/documentation_acceptance.sh` 通过。

## 证据

- archive:`.ai/work-items/archive/WI-681-p1-invariant-coverage-mapping.contract.json`
- verification:`.ai/evidence/WI-681-p1-invariant-coverage-mapping.verification.json`
- finalization:`.ai/decisions/WI-681-p1-invariant-coverage-mapping.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-681-p1-invariant-coverage-mapping.close.json`(待合并后生成)

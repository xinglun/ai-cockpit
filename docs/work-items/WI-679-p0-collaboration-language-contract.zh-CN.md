---
author: AI Cockpit maintainers
title: "WI-679——P0 协作语言契约"
description: "将七个人-Agent 交流节点映射到既有 Runtime 事实的纯文档横向索引，并附十条可检查的语义不变量。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-679-p0-collaboration-language-contract
status: implemented
authority: authorized
lastVerifiedBy: WI-679-p0-collaboration-language-contract
terminalArchive: .ai/work-items/archive/WI-679-p0-collaboration-language-contract.contract.json
terminalVerification: .ai/evidence/WI-679-p0-collaboration-language-contract.verification.json
terminalFinalization: .ai/decisions/WI-679-p0-collaboration-language-contract.finalize.json
terminalDecision: .ai/decisions/WI-679-p0-collaboration-language-contract.close.json
---

[English](WI-679-p0-collaboration-language-contract.md) · [日本語](WI-679-p0-collaboration-language-contract.ja.md)

# WI-679——P0 协作语言契约

## 意图

为无法当面向维护者求助的新用户或贡献者，提供一份横向索引，将仓库真实的交流
节点(任务开始/范围确认、授权请求、验证、阻断/恢复、合并确认、Outcome、
Agent/会话交接)映射到既有 Runtime 事实，依据仓库所有者的明确授权,通过 Work
Item 流程推进 AI Cockpit 协作语言专项。

## 边界

这是仅限文档的 Work Item。新增
`docs/reference/collaboration-language-contract.md`(+ zh-CN/ja)以及
`docs/reference/README.md`(+ zh-CN/ja)中的一条索引条目。不改变任何
CLI、MCP、Contract schema、Outcome schema 或生命周期行为,也不改变任何既有
Outcome 标记、决定状态颜色、Receipt/授权复用规则或阻断/恢复用语的含义——
只引用并索引已经定义它们的来源(`.ai/glossary.md`、
`docs/protocol/v1/specification.md`、`docs/reference/outcome-report.md`、
`docs/reference/how-to-read-cockpit-status.md`、
`docs/reference/agent-workflow.md`、`docs/reference/troubleshooting.md`)。
按照指示,明确排除外部参与者招募、邀请、访谈和评估。状态/转换场景矩阵、
自动化不变量检查、端到端一致性验证和交接完整性检查,在交付文档中被列为后续
Work Item,而不是宣称已经交付。

## 验收与生命周期

- 交付的文档覆盖七个交流节点,并陈述十条语义不变量,每条都注明当前已经成立
  的依据,并明确指出尚不存在自动化检查的缺口。
- `bash tests/docs/documentation_acceptance.sh` 对新增和修改的文件通过。
- 遵循 `start → preflight → checkpoint → verify → finish → archive → close`。

## 证据

- archive:`.ai/work-items/archive/WI-679-p0-collaboration-language-contract.contract.json`
- verification:`.ai/evidence/WI-679-p0-collaboration-language-contract.verification.json`
- finalization:`.ai/decisions/WI-679-p0-collaboration-language-contract.finalize.json`(待合并后生成)
- close:`.ai/decisions/WI-679-p0-collaboration-language-contract.close.json`(待合并后生成)

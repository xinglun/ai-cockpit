---
author: AI Cockpit maintainers
title: "WI-855——已选 successor lineage 恢复"
description: "在不改写历史 evidence 或创建竞争 successor 的前提下，恢复已经选定的多段 successor lineage。"
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
workItemId: WI-855-selected-successor-lineage-recovery
lastVerifiedBy: WI-855-selected-successor-lineage-recovery
terminalArchive: .ai/work-items/archive/WI-855-selected-successor-lineage-recovery.contract.json
terminalVerification: .ai/evidence/WI-855-selected-successor-lineage-recovery.verification.json
terminalDecision: .ai/decisions/WI-855-selected-successor-lineage-recovery.close.json
---

[English](WI-855-selected-successor-lineage-recovery.md) · [日本語](WI-855-selected-successor-lineage-recovery.ja.md)

# WI-855——已选 successor lineage 恢复

WI-855 增加针对已经选定的多段 successor lineage 的 append-only 恢复 receipt。
每条相邻 edge 和每个归档节点都绑定准确的 repository bytes、Runtime identity、
provider PR/finalization 事实及明确的人类决定。无效、过期、foreign、分叉或
歧义历史继续 fail closed；该入口不会创建竞争 successor，也不会改写历史记录。

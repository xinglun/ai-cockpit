---
author: AI Cockpit maintainers
title: "WI-593——WI-592 parity 恢复重验证"
description: "通过追加式后继任务重新交付缺失的 parity 登记，不重写 WI-592 历史。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: canonical
workItemId: WI-593-wi591-doc-parity-recovery
lastVerifiedBy: WI-593-wi591-doc-parity-recovery
terminalArchive: .ai/work-items/archive/WI-593-wi591-doc-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-593-wi591-doc-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-593-wi591-doc-parity-recovery.finalize.f1adebd4711b32d6c621eb97a93219fe741ed770d738229ff4e65c712b470b4b.json
terminalDecision: .ai/decisions/WI-593-wi591-doc-parity-recovery.close.json
---

[English](WI-593-wi591-doc-parity-recovery.md) · [日本語](WI-593-wi591-doc-parity-recovery.ja.md)

# WI-593——WI-592 parity 恢复重验证

## 目标

根据 WI-592 recovery decision 重新交付三语 parity 登记并生成当前验证证据。
WI-592 的归档、Contract、Summary、Outcome 和历史 verification 字节保持不可变。

## 边界

本后继任务只修改三份 reference-parity 投影及自身文档/证据。Runtime 行为、发布制品、
对象工程、全局 Agent/MCP 配置和 WI-592 不可变字节均不在范围内。

## 验收

1. 在最新 reviewed `main` 上三语 parity gate 全部通过，且不重写 WI-592 归档/证据字节。
2. Recovery decision 继续绑定 WI-592 的 repository identity 和不可变摘要。
3. 验证与文档输出不制造未经支持的完成或治理结论。

## 验证

使用显式 repository context 运行 `cargo test --locked --workspace`、
`tests/docs/parity_status_check.sh`、
`python3 tests/docs/work_item_status_consistency.py --repo <repository>` 和
`tests/docs/documentation_acceptance.sh`。

---
author: AI Cockpit maintainers
title: "WI-560——WI-559 终态文档晋级"
description: "晋级已关闭 WI-559 的文档投影，并登记这个有界的自投影。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-560-doc-promotion-wi559
lastVerifiedBy: WI-560-doc-promotion-wi559
terminalArchive: .ai/work-items/archive/WI-560-doc-promotion-wi559.contract.json
terminalVerification: .ai/evidence/WI-560-doc-promotion-wi559.verification.json
terminalFinalization: .ai/decisions/WI-560-doc-promotion-wi559.finalize.json
terminalDecision: .ai/decisions/WI-560-doc-promotion-wi559.close.json
---

[English](WI-560-doc-promotion-wi559.md) · [日本語](WI-560-doc-promotion-wi559.ja.md)

# WI-560——WI-559 终态文档晋级

## 目标

仅依据不可变的终态记录，晋级 WI-559 的三语 Work Item 页面和
reference-parity 投影。

## 范围与边界

范围限定为 WI-559 的三语页面、三个对应的 reference-parity 页面，以及
本有界自投影的三语页面。只有晋级辅助脚本可以写入终态状态。Runtime 行为、
对象工程、全局 Agent/MCP 配置、源台账语义和无关文档保持不变。

## 验收

- WI-559 的投影引用终态 archive、verification、finalization 和 close，且不改变治理事实。
- 本 Work Item 在自身验证关闭前，以要求的预归档状态登记在三语 parity 页面中。
- 已关闭 Work Item 晋级检查、文档验收、parity 门和声明的验证命令全部通过。
- 不修改不可变 receipt 或无关投影。

## 验证

- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

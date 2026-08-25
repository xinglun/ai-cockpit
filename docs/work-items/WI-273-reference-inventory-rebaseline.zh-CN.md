---
author: AI Cockpit maintainers
title: "WI-273——参考源 inventory rebaseline"
workItemId: WI-273-reference-inventory-rebaseline
description: "在不改变 Runtime 行为的前提下，将逐文件参考源比较台账重新绑定到已审阅的当前默认分支提交。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-273-reference-inventory-rebaseline
terminalArchive: .ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json
terminalVerification: .ai/evidence/WI-273-reference-inventory-rebaseline.verification.json
terminalFinalization: .ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json
terminalDecision: .ai/decisions/WI-273-reference-inventory-rebaseline.close.json
authority: canonical
---

# WI-273——参考源 inventory rebaseline

## 意图

在进入下一语义比较批次前，将逐文件参考源比较台账和面向读者的文档重新绑定到已审阅的
`origin/main` 提交 `487f019`。这是 metadata 与文档事实更新，不是 Runtime 功能变更。

## 范围

- 更新 inventory 的 target commit 及 tracked/working-tree 派生 metadata。
- 保留所有既有分类，包括 WI-270/WI-272 记录和四个明确的 capability/profile migrate gap。
- 保持 deferred 路径为 deferred；metadata 刷新不得关闭语义比较工作。
- 同步英文、简体中文和日文的 comparison 与 parity 文档。
- 保持历史 `docs/work-items/**` 和生成的 evidence 不可变。

## 边界

本 Work Item 不修改 Rust Runtime、CI workflow、Agent/MCP 全局配置或参考源行为，不提前提升
任何 deferred 路径，也不改写已归档 Work Item evidence。归档、verification、decision 等
生成记录由安装版 Runtime 生成，不手工编辑。

## 验收

- inventory target commit、tracked/working-tree 数量和 path digest 与干净的
  `origin/main` `487f01970c49e2b85d17b0cb0536f9d60c8f05e0` 一致。
- 台账包含 5,119 条记录：4,262 条 generated-history、163 条
  implemented-different-by-design、1 条 implemented-equivalent、689 条
  deferred-next-batch 和 4 条 migrate-gap。
- generator 与回归检查拒绝过期 target revision，并校验当前 metadata。
- 三种语言的 comparison 与 parity 文档使用相同 baseline 和数量。
- 文档、inventory、governance 和完整必需检查通过，且不改变 Runtime 业务行为。

## 验证

- 所有 repository-bound 命令都显式使用 `--repo` 的安装版 Runtime。
- `bash tests/conformance/reference_file_inventory_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo . --report <report>`
- Contract 要求的完整 workspace quality 与 hosted checks。

## 终态证据

终态路径由安装版 Runtime 按 Contract 记录：

- Archive：`.ai/work-items/archive/WI-273-reference-inventory-rebaseline.contract.json`
- Verification：`.ai/evidence/WI-273-reference-inventory-rebaseline.verification.json`
- Finalization：`.ai/decisions/WI-273-reference-inventory-rebaseline.finalize.json`
- Close：`.ai/decisions/WI-273-reference-inventory-rebaseline.close.json`

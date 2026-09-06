---
author: AI Cockpit maintainers
title: "WI-614——首次 direct merge 恢复往返"
description: "让无 PR 历史合并的恢复计划生成完整且 fail-closed 的首次收尾收据。"
audience:
  - adopter
  - maintainer
  - reviewer
workItemId: WI-614-direct-merge-no-pr-apply
status: implemented
authority: canonical
lastVerifiedBy: WI-614-direct-merge-no-pr-apply
terminalArchive: .ai/work-items/archive/WI-614-direct-merge-no-pr-apply.contract.json
terminalVerification: .ai/evidence/WI-614-direct-merge-no-pr-apply.verification.json
terminalFinalization: .ai/decisions/WI-614-direct-merge-no-pr-apply.finalize.json
terminalDecision: .ai/decisions/WI-614-direct-merge-no-pr-apply.close.json
---

# WI-614——首次 direct merge 恢复往返

## 目标

让 `work-item finalize-recovery-plan --merge-commit <sha>` 输出完整、类型化的
`direct_merge_no_pr` 首次收据。人只需补充授权字段；Git parents、repository
identity、Contract 绑定、Runtime identity 与历史低 assurance 的清理未知会保持
确定性和可审计。

## 边界

计划不会虚构 PR、提升历史 assurance 或重写 `.ai/` 历史。缺失或矛盾的合并事实
继续 fail-closed。对象工程仍是外部只读验收边界。

## 验证

Runtime lifecycle evidence 覆盖 plan→apply 往返、严格负向场景、workspace tests、
clippy、文档、治理完整性和参考清单检查。终态证据由 Runtime 生成并记录在本 WI
的 archive 与 receipts 中。

[English](WI-614-direct-merge-no-pr-apply.md) · [日本語](WI-614-direct-merge-no-pr-apply.ja.md)

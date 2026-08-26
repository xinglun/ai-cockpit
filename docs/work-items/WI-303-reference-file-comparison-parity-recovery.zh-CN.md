---
author: AI Cockpit maintainers
title: "WI-303——参考文件对比 parity 恢复"
workItemId: WI-303-reference-file-comparison-parity-recovery
description: "在不改写 predecessor 记录的前提下，恢复不可变 WI-302 对比交付缺失的三语 parity 注册。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-303-reference-file-comparison-parity-recovery
terminalArchive: .ai/work-items/archive/WI-303-reference-file-comparison-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-303-reference-file-comparison-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.finalize.json
terminalDecision: .ai/decisions/WI-303-reference-file-comparison-parity-recovery.close.json
authority: canonical
---

# WI-303——参考文件对比 parity 恢复

## 意图

WI-302 是已合并且不可变的对比交付。合并后直接提升 pending parity bridge
会违反 finalization append 边界，因此本 successor 记录 recovery decision，
在三份 parity 投影中真实标记 WI-302 已恢复，并移除过期的 pending registry。

## 范围与边界

只修改 `docs/reference/reference-parity*`、typed pending parity registry 和
本 Work Item 的三份可读文档。WI-302 的 archive、verification、finalization、
recovery 与 merge-observation bytes 全部保持不变。不修改 Runtime、CLI、CI、
release、adopter 或全局 Agent/MCP 行为。

## 验收与验证

- 三份 parity 文档各有一条带不可变 predecessor 与 recovery evidence 的 WI-302
  已恢复行，以及一条 verification 前注册的 WI-303 行。
- 原子恢复投影完成后 pending registry 为空。
- 使用安装版 Runtime 通过 repository-bound lifecycle 与文档/governance gate，
  生成绑定当前 repository 的新 verification receipt。
- hosted checks 通过后再合并；finalization、精确 cleanup 与 close 均由 Runtime 绑定。

---
author: AI Cockpit maintainers
title: "WI-275——参考源 inventory finalization 恢复"
workItemId: WI-275-reference-inventory-rebaseline-finalization-recovery
description: "保留 WI-274 不可变的 stale-finalization 失败后，重新交付有界的逐文件参考源 inventory。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-275-reference-inventory-rebaseline-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.6447db8eaff82a97764a341b733710a51f6574664c28398b40f2026c52f4469b.json
terminalDecision: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json
authority: canonical
---

# WI-275——参考源 inventory finalization 恢复

## 意图

在 `origin/main@487f01970c49e2b85d17b0cb0536f9d60c8f05e0` 上重新建立逐文件、机器可读的参考源比对基线。WI-274 的 pre-merge finalization 在最终文档修正之前生成，绑定了 stale head，因此作为不可变 predecessor 保留。

## 范围

- 将 inventory 元数据、路径摘要和文档计数重新绑定到同步后的默认分支。
- 保留 WI-274 的不可变失败与恢复 lineage，不改写其历史。
- 在任何生成的 verification evidence 之前提交 parity registration。
- 保持英文、中文、日文比对和 parity 文档同步。
- 仅在最终提交稳定后记录 provider finalization。

## 边界

本恢复不改写 WI-274 历史、不放宽治理门、不增加 Runtime 行为、不改变 CI 架构，也不执行延期的架构清洁；仅处理参考源 inventory 比对批次。

## 验收

- inventory 元数据和路径摘要匹配固定目标提交。
- WI-274 不可变 evidence 与 successor 关系可审计。
- parity prearchive 行在 verification evidence 之前提交，且治理门证明该顺序。
- 三种语言文档使用相同基线和计数。
- inventory、文档、治理、workspace、hosted、finalization 和 cleanup 检查通过。

## 验证

- 使用显式 `--repo` 的安装版 Runtime
- reference inventory 与文档验收脚本
- repository governance 和 release policy gates
- `cargo test --locked --workspace`
- hosted PR checks 及 finalization/cleanup evidence

## 终态证据（计划）

- Archive：`.ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json`
- Finalization：`.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.json`
- Close：`.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json`

---
author: AI Cockpit maintainers
title: "WI-271——WI-270 finalization 恢复"
workItemId: WI-271-finalization-recovery
description: "不改写不可变 archive，恢复 WI-270，并在 verification/archive 前绑定 reviewed PR context。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-271-finalization-recovery
authority: canonical
---

# WI-271——WI-270 finalization 恢复

## 意图

WI-270 完成了首个有界的参考源 Contract 比对，但在
`finalize-plan` 仍绑定 provisional `pullRequest: pending` context 时就完成了
archive。Hosted governance 正确拒绝了缺少有效 finalization boundary 的交付。
本 successor 保留 WI-270 的每一个 byte，并在 verification/archive 前绑定真实 PR
context，完成 reviewed delivery。

## 范围

- 精确保留 WI-270 archive、evidence、preflight receipt、文档和 inventory；不得
  改写或删除 predecessor bytes。
- 记录 Runtime 有效的 WI-270 successor recovery decision。
- 在 WI-271 archive evidence 生成前，在三语 parity ledger 中标记 WI-270 已恢复并
  登记 WI-271。
- 创建 reviewed PR，使用准确 URL 执行 `finalize-plan`，再执行安装版 Runtime
  lifecycle 与 hosted checks。
- 完成 merge observation、精确 branch/worktree 清理、finalization verification、
  structured close 和可见的人类 Outcome。

## 边界

这是一个窄范围 lifecycle recovery。不会继续比对新的参考源 slice，不会改写历史
evidence，不会重构 Runtime 的大源文件，也不会修改全局 Agent/MCP 配置。架构清洁
留到参考源比对批次完成后另行划界和验证。

## 验证

- 使用显式 `--repo` 的安装版 Runtime
- governance integrity、parity、inventory 和文档检查
- hosted quality、Windows 和 reference-oracle checks
- finalization 与精确清理 receipt
- 可见 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴`，包含状态、未知项、证据、
  人工决定和下一步

## 终态证据

- Archive：`.ai/work-items/archive/WI-271-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-271-finalization-recovery.verification.json`
- Recovery：`.ai/decisions/WI-270-reference-contract-batch.recovery.json`
- Finalization：`.ai/decisions/WI-271-finalization-recovery.finalize.json`、
  `.ai/decisions/WI-271-finalization-recovery.finalize.e1afe79cf257e0675123913123a2eca1aba0b7bf7ffa85893d0054409b76a258.json`、
  `.ai/decisions/WI-271-finalization-recovery.finalize.3fbc88f554e5c352127cb3872f4e082effd03d1ce8534bd87796be2862252152.json`
- Close：`.ai/decisions/WI-271-finalization-recovery.close.json`
- Reviewed 资源：PR [#224](https://github.com/xinglun/ai-cockpit/pull/224) 与合并观察 PR
  [#225](https://github.com/xinglun/ai-cockpit/pull/225) 均已在托管检查通过后合并。

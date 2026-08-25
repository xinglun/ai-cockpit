---
author: AI Cockpit maintainers
title: "WI-263——WI-260 合并后收尾 reconciliation"
workItemId: WI-263-wi260-reconciliation
description: "保留 WI-260 不可变事实，并通过正确绑定的 successor 恢复合并后的资源边界。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-263-wi260-reconciliation
authority: canonical
---

# WI-263——WI-260 合并后收尾 reconciliation

## 意图

在不改写 WI-260 不可变 archive、verification evidence、blocked pre-merge
finalization root 或历史 Outcome 的前提下，完成其合并后资源边界的
reconciliation。

## 已观察边界

PR #212 已在 reviewed feature head
`84b159d06038b16bbb4a3eae3c1252765c144efb` 合并，merge commit 为
`5e426413f08ed54fe54029e0b910056aa4dceba2`。在独立确认合并后，准确的
`codex/wi-260-recovery-gate` 干净 worktree 以及对应 local/remote branch
已删除。

已安装 Runtime v0.2.31 正确拒绝了将 WI-260 不可变 receipt head 从
`d81475e` 推进到 `84b159d` 的 sequence-1 transition：两者之间包含普通
实现和文档变更，并非只追加允许的 finalization receipt。该拒绝作为
fail-closed 边界保留；不会伪造一个 finalization transition。

Runtime 已生成
`.ai/decisions/WI-260-recovery-gate.recovery.json`，绑定 predecessor 的
Contract/Summary/Outcome/Events，并指向 successor
`WI-263-wi260-reconciliation`。WI-260 仍是不可变历史事实；WI-263 负责
正确绑定的 successor 生命周期及其独立 finalization chain。

## 验收边界

- WI-260 的 archive、verification、Outcome、Events、Summary、Contract 与
  canonical blocked finalization receipt 保持字节不变。
- recovery receipt 由 Runtime 生成并绑定身份，记录旧 receipt 不能跨越
  非 append-only head drift 的原因。
- 文档记录 PR #212、reviewed head `84b159d`、merge commit
  `5e426413` 与准确 branch/worktree cleanup 这些已观察的 provider/resource
  事实。
- WI-263 在 verification/archive 前用 `finalize-plan` 绑定自己的 reviewed
  PR context，并在 close 前记录有效的 finalization chain。
- 英语、简体中文、日语 parity 行区分已恢复的 WI-260 历史与进行中的
  WI-263 successor。

## 验证

- `ai-cockpit inspect/status/doctor/agent doctor --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check_test.sh`

## 证据边界

Recovery 是历史投影，不表示 WI-260 的旧 finalization chain 已被 Runtime
判定为绿色。只有 successor 新 Contract、verification evidence、provider
finalization 和结构化人类决定，才能建立当前终态边界。

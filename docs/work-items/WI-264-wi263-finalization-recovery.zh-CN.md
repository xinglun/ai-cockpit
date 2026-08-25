---
author: AI Cockpit maintainers
title: "WI-264——WI-263 finalization recovery"
workItemId: WI-264-wi263-finalization-recovery
description: "在不改写 predecessor 不可变 bytes 的前提下恢复已合并 WI-263 的资源边界。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-264-wi263-finalization-recovery
authority: canonical
---

# WI-264——WI-263 finalization recovery

## 意图

已安装 Runtime 拒绝了一个 predecessor head 已过期的 post-merge
finalization transition。本 Work Item 记录有边界的 successor，保留 WI-263
历史事实，并只负责准确的资源清理边界。

## 已观察边界

PR #215 已合并，merge commit 为
`47c9dd8e7107526f280274a92ccc7399493125cb`，reviewed feature head 为
`ce7af9def1ccf4066eded50f56d1a5b301f1ca8b`。WI-263 的不可变 pre-merge
finalization root 仍绑定
`bc8f8e655a7616965b06ddacbc0feb0c807e64a0`。中间包含文档修正，Runtime
正确拒绝把它伪装成只追加 finalization receipt。

Runtime 生成的
`.ai/decisions/WI-263-wi260-reconciliation.recovery.json` supersede 了
WI-263，但不会改写其 archive、evidence、Outcome、events 或 finalization
root。

## 验收边界

- WI-263 历史 bytes 保持字节不变。
- recovery receipt 绑定 predecessor digests 与 Runtime identity。
- 合并 PR head 与 merge commit 作为 provider facts 被记录。
- 只有在 Runtime finalization receipt 和本地 postcondition 验证后，才删除
  准确的 branch 与 worktree。
- 英语、简体中文、日语文档保持一致。

## 验证

- 使用显式 `--repo` 的已安装 Runtime `inspect`、`status`、`doctor`。
- 绑定本 Work Item 的 Runtime verification。
- Provider merge 与准确 branch/worktree cleanup 检查。
- `tests/ci/governance_integrity_gate_test.sh`。
- 文档 parity 与 acceptance 检查。

## 证据边界

本 recovery 不会让 WI-263 过期的 finalization chain 变绿，也不会改写历史
记录。只有本 Work Item 新鲜的 verification、provider receipt、archive 和
结构化 close decision 才能建立当前终态边界。

---
author: AI Cockpit maintainers
title: "WI-188——治理完整性门禁"
description: "动态、失败关闭地盘点当前 Work Item、证据、决定、Outcome、文档与 CI 覆盖。"
audience:
  - maintainer
  - reviewer
workItemId: WI-188-governance-integrity-gate
status: implemented
authority: canonical
lastVerifiedBy: WI-188-governance-integrity-gate
---

# WI-188——治理完整性门禁

WI-188 用 repository 动态盘点替代固定的 WI-177 至 WI-186 列表。门禁自动发现
active 与 archived Contract，从 Cargo metadata 以及 Contract/archive 创建时间推导当前发布
周期，并检查当前 Summary、archive、verification、终态 decision、Outcome 与三语
parity 绑定。更早的记录仍以 historical 或 legacy 项可审计地呈现；未知的当前问题
会失败关闭。

只有当 Runtime finalize 回执证明 PR 尚未合并、分支仍存在、worktree 已干净、
result disposition 为 `blocked`、唯一 failure code 为 `unmerged_pull_request`、
unknown code 为空且 reason 以 `awaiting_merge_close` 审计 token 开头时，feature 分支上已 archive 的
Work Item 才可标记为 `awaiting_merge_close`。该回执不是终态关闭；进入默认分支后，
门禁仍必须看到精确的 close 或 recovery decision。
该例外还会绑定 repository identity、archived Contract 原始 SHA-256、verification Runtime
identity、实际远程默认分支，并要求 PR、branch、worktree 与 Contract resource context
的身份完全一致。

CI 通过单一 manifest 执行 documentation、workflow、conformance、performance 与
release 门禁。Workspace package 测试从 `cargo metadata` 动态派生、串行执行，
并绑定到确定性的 JSON 回执。

[English](WI-188-governance-integrity-gate.md) ·
[日本語](WI-188-governance-integrity-gate.ja.md)

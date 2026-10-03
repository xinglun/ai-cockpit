---
author: AI Cockpit maintainers
workItemId: WI-1055-v1-0-1-bounded-hotfix
title: 有界修复版例外记录
description: 记录 WI-1052 与 issue 1004 的有界修复版范围、Runtime 激活拒绝、实际测试及待完成的发布证据。
audience: [maintainer, reviewer]
status: in-progress
authority: user-authorized-project-bootstrap-exception
lastVerifiedBy: pending-exact-head-acceptance
---

[English](WI-1055-v1-0-1-bounded-hotfix.md) · [日本語](WI-1055-v1-0-1-bounded-hotfix.ja.md)

# WI-1055 有界修复版例外记录

状态：按用户授权的项目级临时 bootstrap 例外实施中；不表示 Runtime 已准入、已验证、已关闭、已合并或已发布。

基线是 main `78ae7240aac016f005fcf4ced61fcb251ea20eb1`。旧候选 `befdbd11d60af5c01d40ec62497aa9c8b46ba0ce` 仅作代码参考，不搬运 WI-1053 的 active Contract、Summary、修订收据或决定。

范围仅含 WI-1052 的待审修订请求生成（绝不自动批准），以及 Issue #1004 的跨 checkout 收尾恢复、历史 Contract 原始字节绑定和原子回滚。Task9、旧 WI 状态追认、全局忽略 `.ai/`、Sentinel 资源变更均不在范围内。

Runtime `1.0.1-rc.1` 生成 WI-1055 骨架后，实际以 `archived_work_item_scope_conflict:WI-1042-contract-amendment-environment-drift, archived_work_item_scope_conflict:WI-1043-amendment-review-admission-fix` 拒绝激活；骨架仍为 `not_ready`。本文不是 Runtime 收据或身份绑定的人类审查。

新分支定向测试已通过：WI-1052 CLI 4/4，#1004 repository 3/3、CLI 5/5、MCP 1/1，repository 库内 54/54。Sentinel formatter 的 scope 修正为精确路径后，即使六份历史恢复文件仍在 Summary.changedPaths，真实 preflight 也不再报 `scope_exceeded`，而是无 blocker 的黄色、等待人类审查。因此不能把此前红态单独归咎于恢复文件。

完整 workspace、PR CI、独立审查、Mac arm64 下载校验／安装／doctor 和公开发布均未完成。不得覆盖 `v1.0.1-rc.1` 的 tag 或资产；若只有 Mac 证据，应使用新的 prerelease，不宣称稳定版或跨平台已验收。

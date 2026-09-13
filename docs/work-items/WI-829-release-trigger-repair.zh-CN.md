---
author: AI Cockpit 维护者
title: "WI-829——显式发布触发修复"
description: "使发布身份绑定并仅通过显式 dispatch 启动。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-829-release-trigger-repair
lastVerifiedBy: WI-829-release-trigger-repair
---

[English](WI-829-release-trigger-repair.md) · [日本語](WI-829-release-trigger-repair.ja.md)

# WI-829——显式发布触发修复

## 意图与边界

本 Work Item 使发布只能通过携带治理 Work Item identity 的显式
`workflow_dispatch` 启动。annotated tag 先作为不可变输入创建并推送；单独推送
tag 不能启动无法解析身份的发布路径。

Runtime 协议、产品行为、历史 Work Item 迁移、已有 Release 或 tag 改写，以及
用户全局配置均不在本 Work Item 范围内。

## 验收

- 缺失、格式错误或有歧义的发布 identity 在编译或发布前被拒绝。
- dispatch 在昂贵发布任务前校验不可变 tag、源码提交、Work Item 和 Contract。
- 候选及公开安装、升级、handoff、attestation 和 close 门禁保持必需。
- 策略及三种语言的发布文档描述同一个仅 dispatch 边界。

## 验证

- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `bash tests/release/version_consistency.sh --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh <repo>`
- `cargo fmt --all --check`
- reviewed PR 的 hosted CI 及一次显式 v0.2.92 发布 dispatch。

## 恢复策略

不可变 tag 和公开 Release 永不改写。失败的 dispatch 只从第一个失效阶段恢复；
只有在输入、Contract、helper 和配置身份仍一致时才复用证据。


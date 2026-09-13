---
author: AI Cockpit 维护者
title: "WI-830——发布 dispatch 语法修复"
description: "修复发布前发现的 release close 表达式错误。"
audience: [maintainer, reviewer]
status: in_progress
authority: authorized
workItemId: WI-830-release-dispatch-syntax
lastVerifiedBy: WI-830-release-dispatch-syntax
---

[English](WI-830-release-dispatch-syntax.md) · [日本語](WI-830-release-dispatch-syntax.ja.md)

# WI-830——发布 dispatch 语法修复

## 意图与边界

WI-830 修复 dispatch-only 发布 close 条件中多出的右括号；该错误在 GitHub
创建运行之前就被拒绝。本 Work Item 同时为这个准确的错误表达式增加本地回归。

Runtime 协议、产品行为、发布验收执行、不可变 tag 或 Release 改写、历史迁移和
用户全局配置均不在范围内。

## 验收

- GitHub 接受 `workflow_dispatch` 发布 workflow。
- 本地 release policy 拒绝错误 close 表达式并接受括号平衡的表达式。
- 本修复不改写产品制品、tag 或 Release。
- 发布重试前 hosted checks 全部通过。

## 验证

- `bash tests/ci/release_gate_policy_test.sh`
- `bash tests/release/workflow_policy.sh .github/workflows/release.yml`
- `bash tests/release/action_runtime_policy.sh .github/workflows/release.yml`
- `cargo fmt --all --check`
- hosted PR checks 及一次显式发布 dispatch。


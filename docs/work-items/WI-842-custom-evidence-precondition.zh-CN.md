---
author: AI Cockpit 维护者
title: "WI-842——自定义证据 verification 前置条件"
description: "在启动任何对象工程 verification 进程前校验仓库绑定的自定义证据。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-842-custom-evidence-precondition
lastVerifiedBy: WI-842-custom-evidence-precondition
---

[English](WI-842-custom-evidence-precondition.md) · [日本語](WI-842-custom-evidence-precondition.ja.md)

# WI-842——自定义证据 verification 前置条件

## 意图与边界

本 Work Item 使 verification 入口在启动对象工程进程前，使用当前 repository、
Contract、文件类型和字节 digest 校验自定义证据。完整 projection 不能被报告层
warning 阻塞；缺失、过期、格式错误、外部仓库、符号链接和非普通文件仍必须
fail-closed。Runtime 全局复用策略、历史证据改写、发布和资源清理不在本范围内。

## 恢复边界

属于当前范围的失败应在当前 Work Item 上 amend 并重新验证。只有 scope、authority
或 base 真正不同、独立变更、无法安全进行的范围内修复、不可变失败交付或明确的
人工指示才使用 successor；successor 必须保留 predecessor 绑定。

## 验收

- 完整且绑定当前 repository 的自定义证据允许通过 verification 前置检查；高风险场景尚未执行但已有计划时也一样。
- 非法自定义证据在启动对象工程进程前被拒绝，并给出稳定诊断。
- 即使正式完成证据被拒绝，执行尝试仍保留退出状态、超时、耗时、日志和输入身份。
- 仅治理修正且执行输入未变时允许复用；执行输入变化必须使复用失效。

## 验证

- `cargo test --locked -p cockpit-repository --test lifecycle_entry`
- `cargo test --locked -p cockpit-repository --test archive_integrity --test verification_attempts`
- `cargo fmt --all -- --check`
- `git diff --check`

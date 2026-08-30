---
author: AI Cockpit maintainers
title: WI-417——确定性的 Cargo 验证脚手架命令选择
description: 根据仓库事实选择可执行的默认 Cargo 验证命令。
workItemId: WI-417-cargo-verification-lock-strategy
audience: [adopter, contributor, maintainer, reviewer]
status: implemented
authority: human-authorized
lastVerifiedBy: WI-417-cargo-verification-lock-strategy
---

# WI-417——确定性的 Cargo 验证脚手架命令选择

[English](WI-417-cargo-verification-lock-strategy.md) · [日本語](WI-417-cargo-verification-lock-strategy.ja.md)

## 意图

让 Cargo Work Item 脚手架激活时生成的默认验证命令在当前仓库中可执行。
存在受版本控制的 `Cargo.lock` 时选择 `cargo test --locked --workspace`；没有
锁文件的 Cargo 仓库选择 `cargo test --workspace`；非 Cargo 仓库不臆造 Cargo 命令。

## 范围与边界

`start` 与 recovery scaffold 激活使用同一确定性规则。本 Work Item 只调整命令
选择及其参考文档，不改变验证语义、发布/adopter harness、Sentinel 源码或全局
Agent/MCP 配置。

## 证据

- Archive：`.ai/work-items/archive/WI-417-cargo-verification-lock-strategy.contract.json`
- Verification：`.ai/evidence/WI-417-cargo-verification-lock-strategy.verification.json`
- Finalization：`.ai/decisions/WI-417-cargo-verification-lock-strategy.finalize.json`
- Close：`.ai/decisions/WI-417-cargo-verification-lock-strategy.close.json`
- 已审查 PR：[ #382](https://github.com/xinglun/ai-cockpit/pull/382)

锁文件、无锁文件和非 Cargo 场景的定向测试，以及 Runtime v0.2.43 下的完整锁定
工作区测试均已通过。

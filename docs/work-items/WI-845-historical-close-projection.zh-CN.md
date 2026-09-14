---
author: AI Cockpit maintainers
title: "WI-845 — 历史 close 投影"
description: "在不改写非规范历史 close 的前提下，投影有效的追加式 successor recovery。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-845-historical-close-projection
lastVerifiedBy: WI-845-historical-close-projection
---

[English](WI-845-historical-close-projection.md) · [日本語](WI-845-historical-close-projection.ja.md)

# WI-845 — 历史 close 投影

## 意图与边界

本 Work Item 使 Runtime 状态投影能够识别已归档 Work Item 的有效追加式
successor recovery，即使其历史 close decision 不符合当前规范。前置项的
原始字节保持不变；只有在 repository-bound recovery、successor、archive 和
close 绑定全部通过校验后，投影才会解除阻塞。Protocol schema、CLI 界面、
发布与 adopter 行为、历史记录改写以及大范围分支清理不在本边界内。

## 恢复边界

同一范围内的缺陷在本 Work Item 中 amend 并重新验证。只有真正不同的范围、
权限或 base、独立变更、不安全的范围内修复、不可变的失败交付，或明确的人类
指示，才使用 successor。无效、不完整、外部、过期或被篡改的 recovery 证据
仍保持阻塞，绝不会投影为普通 closed。

## 验收

- 有效的 repository-bound successor recovery 且 successor 已终态时，前置项
  投影为 recovered 且不阻塞，同时不改写前置 close 字节。
- 未 close、不匹配、格式错误、外部或被篡改的 successor recovery 仍保持阻塞，
  不绕过 close 校验。
- Rust 聚焦回归测试覆盖正向投影及 fail-closed 负向案例。
- 实现不产生部分状态写入，并保留原始历史 close 字节。
- PR 自带所需的预归档三语 Work Item 页面及本 Work Item 唯一 parity 行，使
  repository quality gate 能在昂贵验证前校验这项变更。

## 验证

- `cargo test --locked -p cockpit-repository --test status_projection`
- `cargo test --locked -p cockpit-repository --test recovery_decision`
- `cargo clippy --locked -p cockpit-repository --all-targets --all-features -- -D warnings`
- `cargo fmt --all -- --check`
- `git diff --check`

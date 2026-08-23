---
author: AI Cockpit maintainers
title: "WI-193——发布验收隔离加固"
description: "让 adopter harness 的清理、源码 manifest 与允许写入根 symlink containment fail closed。"
audience:
  - maintainer
  - reviewer
workItemId: WI-193-release-acceptance-isolation
status: historical
authority: canonical
lastVerifiedBy: WI-195-governance-recovery-gate
---

# WI-193——发布验收隔离加固

WI-193 是不可变的历史 predecessor。由于无法在正确的已发布 Runtime context 刷新生命周期回执，predecessor 保持红色/blocked，绝不显示为绿色完成；当前有界交付由 WI-195 继续。

WI-193 在两个 adopter harness 创建临时 run root 之前安装 EXIT 清理处理器。
因此 toolchain 解析或 setup 失败也会生成带 checksum 的 `acceptance.json` 与
`cleanup.json` 回执，并且不留下 run root。

源码隔离现在比较确定性的 typed manifest，覆盖所有 tracked/untracked 源码路径及
全部 `.ai` 条目，包括 ignored 内容。只排除声明的 output 子树；同时规范化 output
祖先目录的 metadata，避免在源码 checkout 内写验收 evidence 产生假 mutation。
TMPDIR 与 CARGO_HOME manifest 保留 symlink metadata、字面 target 和解析后的 target，
并拒绝任何越过对应允许写入根的 target。

已提交的 v0.2.23 公开 adopter 与 v0.2.22 → v0.2.23 N-1 回执都明确记录
`aarch64-apple-darwin`。Linux x86_64 仍是 Release workflow 的 CI 覆盖，不是
第二个完整 adopter evidence target。本 Work Item 不改写公开 Release、tag、历史
evidence、Runtime Core、crates 或 CI parity 文件。不可变恢复回执见
[WI-193 recovery](../../.ai/decisions/WI-193-release-acceptance-isolation.recovery.json)。

[English](WI-193-release-acceptance-isolation.md) ·
[日本語](WI-193-release-acceptance-isolation.ja.md)

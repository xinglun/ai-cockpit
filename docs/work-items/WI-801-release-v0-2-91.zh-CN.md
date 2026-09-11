---
author: AI Cockpit maintainers
title: "WI-801——受治理的 v0.2.91 发布"
description: "在 WI-799 根因修复之后准备并发布 ai-cockpit v0.2.91。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-root-cause-repair-and-release
workItemId: WI-801-release-v0-2-91
lastVerifiedBy: WI-801-release-v0-2-91
---

[English](WI-801-release-v0-2-91.md) · [日本語](WI-801-release-v0-2-91.ja.md)

# WI-801——受治理的 v0.2.91 发布

## 意图与边界

WI-801 将 Cargo workspace 和 lockfile 对齐到 v0.2.91，登记三语治理投影，并将
经过评审的 main 修订发布为不可变 Release。发布 workflow 必须使用候选制品，以及
下载的公开制品完成安装和 N-1 升级验收。

本 Work Item 不改变 Runtime 行为、发布 workflow 语义、历史记录、已有 Release 或
标签，也不修改无关源码特性。

## 验收

- `Cargo.toml` 和 `Cargo.lock` 一致标识 v0.2.91，且没有无关依赖变化。
- 三个 WI-801 页面和三语 parity 行在验证前完成登记，并保持语义一致。
- hosted checks 通过后，经过评审的 main 才能打标签并发布 v0.2.91。
- 候选全新安装与 N-1 升级，以及公开全新安装、公开 N-1 升级、版本一致性和 release
  close，均绑定不可变公开制品并证明隔离清理。

## 验证

- `cargo test --locked --workspace`
- `bash tests/release/version_consistency_test.sh`
- `bash tests/docs/parity_status_check.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `python3 tests/ci/governance_integrity_gate.py --repo <repo>`
- hosted release preflight、source quality、候选验收、公开制品验收、N-1 升级验收和
  release close。

## 证据规则

发布标签、Release 制品、workflow receipt 和公开 adopter receipt 都是不可变的外部事实。
Runtime 所有的 Contract、Summary、Outcome、verification、archive、finalization 和
close 记录均由安装版 Runtime 生成。移动分支、源码 checkout 或 workspace binary
不能替代公开制品。

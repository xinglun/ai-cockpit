---
author: AI Cockpit maintainers
title: "WI-236——v0.2.30 发布基线与 public adopter 验收"
workItemId: WI-236-release-v0-2-30
description: "从已合并默认分支发布 v0.2.30，并用安装后的 Runtime 验证不可变公开 artifact。"
audience:
  - adopter
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-236-release-v0-2-30
---

# WI-236——v0.2.30 发布基线与 public adopter 验收

本 Work Item 建立下一份不可变公开 Runtime Release 的发布前基线。先统一
package identity 与面向读者的发布文档，再绑定已审阅 PR 与合并前最终化边界。
公开 artifact identity、安装后的 binary、adopter lifecycle 与 N-1 upgrade
都属于合并后的事实，由 successor Work Item 在发布后验收；本 Work Item 不会
提前宣称这些事实。

## 验收边界

- workspace metadata 与 `Cargo.lock` 一致标识 v0.2.30。
- 发布、版本、分发以及中/英/日 parity 文档标识 v0.2.30，并把 v0.2.29
  作为紧邻的 N-1 基线。
- 发布前 source quality、release policy、version consistency 与文档 gate 通过。
- 已审阅 PR 具备有效的合并前最终化边界；只有合并后才能创建公开 Release tag。
  不接受源码 checkout 或 workspace binary 作为公开发布证据。
- successor Work Item 在发布后验证安装的 v0.2.30 binary 与隔离 adopter/upgrade
  harness；临时运行根目录必须清理，但 successor 的验收 receipt 保持可审计。

## 参考

- [发布与分发](../release/distribution.zh-CN.md)
- [版本管理](../architecture/versioning.zh-CN.md)
- [参考 parity ledger](../reference/reference-parity.zh-CN.md)
- [public adopter 验收 harness](../../tests/release/adopter_acceptance.sh)

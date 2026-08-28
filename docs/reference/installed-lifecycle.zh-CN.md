---
author: AI Cockpit 维护者
title: 已安装 Runtime 生命周期
description: 安装、repository attach、升级、回滚和卸载的边界。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/installed-lifecycle.md
lastVerifiedBy: WI-347-reference-knowledge-trust-lifecycle-assessment
capabilityClaims:
  - shared_runtime_lifecycle
---

# 已安装 Runtime 生命周期

[English](installed-lifecycle.md) · [简体中文](installed-lifecycle.zh-CN.md) · [日本語](installed-lifecycle.ja.md)

安装会在机器上放置一份共享的 `ai-cockpit` Runtime，不会自动 attach repository、选择工程或证明每条生命周期路径都完成。Attach 必须显式执行：

```text
ai-cockpit attach --repo /path/to/repository
ai-cockpit inspect --repo /path/to/repository
ai-cockpit doctor --repo /path/to/repository
```

仓库拥有 `.ai/cockpit.toml`、Contract、证据、Knowledge 和 adapter 记录。Runtime 没有持久 current repository 或全局 active Work Item。

## Release 与仓库边界

安装和升级必须使用命名的不可变公开 Release archive 及其 SHA-256/manifest。Release 分发、Homebrew、SBOM、provenance、回滚和发布后 adopter 验收见[Release 与分发](../release/distribution.zh-CN.md)，属于仓库本地 Contract 之外的边界。移动分支或 workspace binary 不是发布证据。

Runtime-only 升级通常不改变仓库字节。Schema migration 是独立、显式、经过审查的操作，需要计划、备份/回滚证据和人工决定；Runtime 升级不会重写历史证据。

卸载同样分为 proposal 和执行边界；除非仓库 owner 明确授权处置，否则保留仓库证据。删除本地 binary 不代表 installer、provider、sandbox 或 enterprise retention 已完成。

## 参考源映射

参考源的 Python installer 阶段、Make target、生成状态和迁移记录是 conformance 资料，不是复制对象。Rust 使用共享 Runtime、显式仓库上下文、类型化回执和公开制品验收 harness；provider/enterprise 操作必须保持外部可验证证据引用。

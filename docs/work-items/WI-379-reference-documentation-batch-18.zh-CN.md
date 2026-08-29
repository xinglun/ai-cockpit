---
author: AI Cockpit maintainers
title: “WI-379——参考源文档第 18 批”
description: “比较固定参考源下一批十个路径，发布有界的 Rust-native 阅读路由。”
workItemId: WI-379-reference-documentation-batch-18
canonical: docs/work-items/WI-379-reference-documentation-batch-18.md
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: translation
lastVerifiedBy: WI-379-reference-documentation-batch-18
capabilityClaims: [reference_comparison, verification_reuse, intelligence, lifecycle_closure]
---

# WI-379——参考源文档第 18 批

[English](WI-379-reference-documentation-batch-18.md) · [简体中文](WI-379-reference-documentation-batch-18.zh-CN.md) · [日本語](WI-379-reference-documentation-batch-18.ja.md)

## 意图

逐一比较固定 inventory 的下一批十个路径，把其面向读者的治理含义映射到共享 Rust
Runtime；不复制源 Python、Make、Provider 配置或历史决定。

## 路径与决定

| 固定路径 | 决定 |
| --- | --- |
| `docs/reference/upgrade.md` | `implemented-different-by-design`；补充 migration、备份/冲突、回滚和 adapter 边界的三语升级路由。 |
| `docs/reference/verification-evidence-reuse-runtime.md` | `implemented-different-by-design`；说明 typed receipt 绑定、受保护节点、planner/adapter 分离和可观测复用。 |
| `docs/reference/verification-evidence-reuse.md` | `implemented-different-by-design`；说明新鲜度、失效和调用次数证据。 |
| `docs/reference/verification-fixture-boundary.md` | `implemented-different-by-design`；说明 Rust fixture 隔离和本地证据限制。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.json` | `reference-only`；历史生成的 V1 audit bytes 不是目标 authority。 |
| `docs/reference/wi01-wi20-bidirectional-traceability-audit.md` | `reference-only`；历史叙述保持源绑定，不复制。 |
| `docs/reference/wiii-v2-integration-audit.md` | `implemented-different-by-design`；说明更窄的 Rust 只读 Intelligence 投影和 identity 检查。 |
| `docs/reference/work-item-intelligence-performance-baseline.md` | `implemented-different-by-design`；说明可复现本地观测，不声称源数字或 SLO。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` | `implemented-different-by-design`；提供完整 Rust-native 三语关闭路由。 |
| `docs/reference/work-item-lifecycle-closure.md` | `implemented-different-by-design`；提供英语路由和明确恢复边界。 |

## 边界

这是语义/文档 parity，不是源命令、JSON-wire、Provider 或生成历史兼容。一份已安装 Runtime
通过显式 `--repo` 服务多个 repository；事实、Work Item、evidence、knowledge 和 snapshot
始终隔离。文档不能生成 authority、approval、assurance 或 verification evidence。

## 验收

- 每个选定路径都有一个 inventory 分类及 counterpart 或明确 reference-only 理由。
- 英语、简体中文、日语路由的链接和语义/非 wire 边界一致。
- inventory 与 parity 记录相同 source commit 和批次决定，`migrate-gap` 为 0。
- 文档、inventory、conformance 和安装版 Runtime 检查通过，且没有源码 fallback。
- presentation localization 不改变 Contract 语言中的治理事实。

## 验证

执行 inventory、inventory-docs、inventory regression、documentation acceptance、status
consistency 以及 `cargo test --locked --workspace`。preflight、checkpoint、verify、finish、
archive、finalize、close 全部使用安装版 v0.2.39；评审合并后才写入终态 receipt。

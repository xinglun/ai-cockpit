---
author: AI Cockpit maintainers
title: "WI-123 — Contract 并行边界与 Slot"
description: "由 Contract 拥有并行路径边界以及 repository-local slot lease。"
audience:
  - adopter
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-123
capabilityClaims:
  - parallel_contract_boundary
---

# WI-123 — Contract 并行边界与 Slot

## 目标

让并行 Work Item 授权显式、按 repository 隔离并 fail closed。Contract 可以
增加包含四类路径、schema、reason 和 `maxWorkers` 的 `concurrencyBoundary`；
既有 intelligence sidecar 继续承担 depends、conflicts 与 `parallelizable` 声明。

## 范围

- additive 的 `ConcurrencyBoundary` 与严格 `ParallelSlotLease` 协议类型；
- 保守比较 exact、prefix、嵌套 glob 以及 Windows 分隔符；
- repository-local 独占 slot reservation 与 lease acquire/release/list；
- CLI `work-item boundary`、`work-item slot` 与 MCP `work_item_parallel`；
- 三语文档及 race regression tests。

## 安全边界

未知或格式错误的边界、路径、lease 会序列化并 fail closed。lease 绑定
repositoryId 和 Work Item，不自动过期，其他 Work Item 不能释放它。`maxWorkers`
是并行 slot 容量，与 `verify --workers` 不同；不创建全局 Agent/MCP 配置或 current repository。

## 兼容性与验证

`concurrencyBoundary` 是可选字段，因此旧 Contract 与 sidecar 仍可读取；没有边界时继续使用既有
scope 比较，一旦任一方声明边界，缺失或不兼容信息就只能作为 unknown。协议 round-trip、严格未知字段、
边界重叠、Windows 分隔符、缺失边界、slot 容量、重复 ID race 以及 repository 隔离均有 Rust 测试覆盖。

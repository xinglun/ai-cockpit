---
author: AI Cockpit maintainers
workItemId: WI-125-contract-schema
title: Contract V2 schema completeness
description: 在不重写历史 bytes 的前提下补齐剩余 typed Contract V2 lineage 与治理字段。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: documentation-acceptance
---

# WI-125 — Contract V2 schema completeness

## 目的

在保持共享 Runtime 与 repository-local Protocol 分离的同时，完成 Rust
Contract 边界，使其能够读取参考 Work Item 模型。本 WI 增加 typed 字段和
确定性的 cross-field 校验，不复制参考工程的 Python runtime 或 Makefile workflow。

## 已交付

- typed 支持 `baseCommit`、`baselineDirtyPaths`、`archiveSequence`、
  `resumeHistory`、`synchronizationCheckpoint`、`synchronizationHistory`、
  `guidelines`、`preReviewWarnings` 和可选 `acceptance`。
- typed 支持包含 identity level、actor、scope、evidence payload 的 repository-local
  authority 与 destructive approval evidence。
- Contract V2 的 mode 限定为 `investigate`、`author_todo`、`code`、`review`、`cleanup`；
  `code` 必须满足 `unknowns` 为空和 `notCodable: false`。
- 对 unknown nested field、malformed lineage、空 path/guideline、未授权 synchronization
  checkpoint、不连续 history 和不充分 approval evidence fail closed。
- protocol-v1 记录、legacy `baseRevision` 与单行 intent 仍可读取；不回写历史 Contract bytes。

## 边界

Summary、WIII、Outcome、evidence strictness、release checks、README、MCP 以及参考工程的
Python/Makefile runtime 不在本 WI 范围。Approval record 只描述 repository provenance，
不认证具体人员，也不替代 provider/enterprise review。

## 验证

- `cargo test --locked -p cockpit-protocol --test contract_v2`
- `cargo test --locked -p cockpit-repository --test contract_schema`
- 合并前必须通过 locked workspace 全量测试和 lint。

面向人的 handoff 必须显示 `Outcome: 🟢`、`Outcome: 🟡` 或 `Outcome: 🔴` 之一，并直接显示
status、unknowns、evidence、human decision 和 next action，不能依赖被折叠的日志。

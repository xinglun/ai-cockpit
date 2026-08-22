---
author: AI Cockpit maintainers
title: "WI-110 — Evidence assurance 与历史投影"
description: "严格验证证据、当前 Runtime 绑定和诚实的 legacy 投影。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: WI-110-evidence-assurance
---

# WI-110 — Evidence assurance 与历史投影

## 意图与目标

明确 verification trust boundary：持久化的 v2 evidence envelope 和 captured
receipt 必须有类型、绑定 identity 并 fail closed。当前 CLI 生命周期只能接受当前已安装
Runtime 生成的 evidence。历史 pre-v2 bytes 保持不可变，并显示为历史输入，而不是伪造当前失败。

## 范围

- 严格的 `VerificationEvidenceV2` envelope 与嵌套 `VerificationReceipt` 校验；
- Work Item、repository、snapshot 和 Runtime identity 绑定；
- Runtime-bound CLI/MCP verify、finish、archive、close 与 Outcome 路径；
- unknown field、嵌套 identity 缺失、malformed、foreign Runtime 和 legacy evidence 回归测试；
- English、简体中文和日本語文档。

## 不变量

未知 envelope 或 captured receipt 字段、嵌套 identity 缺失、无效 digest 和 foreign Runtime
identity 不能产生绿色 Outcome，也不能通过 Runtime-bound 生命周期。`digest_only` 保留模式
本来就没有 captured receipt。pre-v2 record（没有 `evidenceSchemaVersion`）是只读历史输入，
投影为黄色 `legacy_evidence_historical`；不会被重写、提升为绿色或报告为当前红色失败。v2
record 若缺少 identity 仍然是红色。

没有显式 `RuntimeContext` 的兼容 Rust API 保留给自行管理 Runtime identity 的 embedder；已安装
CLI 和 repository-bound MCP 始终使用 Runtime-bound API。

## 验证

Focused evidence/lifecycle 测试覆盖严格 envelope/嵌套 receipt 篡改、foreign Runtime 拒绝、CLI
foreign-runtime 拒绝和不可变 legacy 投影。合并前必须通过 workspace format、Clippy 和完整测试。

## 边界

本 Work Item 不实现 provider attestation、外部不可变审计存储或历史 bytes migration；这些属于
独立的企业 assurance 工作。

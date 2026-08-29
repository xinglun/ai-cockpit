---
author: AI Cockpit maintainers
title: Work Item Intelligence 集成边界
description: 不宣称源 wire 兼容的 Rust-native、可审计 Work Item Intelligence 投影。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: translation
canonical: docs/reference/wiii-v2-integration-audit.md
lastVerifiedBy: WI-379-reference-documentation-batch-18
---

# Work Item Intelligence 集成边界

[English](wiii-v2-integration-audit.md) · [简体中文](wiii-v2-integration-audit.zh-CN.md) · [日本語](wiii-v2-integration-audit.ja.md)

Rust Runtime 提供 request-scoped、只读的 Work Item Intelligence 投影。schema 版本
保持显式；source-bound 不一致会报告 `inconsistent`，不会静默重建；不会调度 Work Item、
调用 provider 或虚构人工批准。

`status` 和 intelligence 命令读取 repository-local records/evidence。V2 投影只有在
显式命令下重建，并按 source identity 校验；损坏或不一致的记录保持 unknown/inconsistent。
查询、分页和 cursor 都绑定显式 `--repo`。

该投影比参考源 Python CLI 更窄，不是直接 JSON/API 兼容。历史评估分数、生成 audit bytes
和 provider 结果仍是 reference-only。共享 Runtime 可以服务多个 repository，但 Work Item、
evidence、knowledge 和 snapshot 始终隔离。

本审计不证明 provider identity、分布式调度、网络隔离、人工批准或企业合规；这些需要独立
policy/provider 边界与证据。

---
author: AI Cockpit maintainers
title: "WI-111 MCP 面向人的 handoff 与发布隔离证据"
description: "repository-bound Outcome 交付与带类型的发布后隔离 manifest。"
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: mcp-isolation-regression
capabilityClaims:
  - mcp_human_outcome_handoff
  - typed_release_isolation_evidence
---

# WI-111：MCP 面向人的 handoff 与发布隔离证据

## 目标

让面向人的 Outcome 成为 Agent 的正式交付路径，并让发布 adopter 的隔离证据能够发现文件、目录、symlink、
metadata 和 digest 的变化，同时保持清理和 repository binding 不被削弱。

## 范围

repository service 提供单一 human Outcome renderer。CLI 和 MCP 都在 `outcome_v2` 校验后调用它。MCP 增加
明确 repository-bound 的 `work_item_outcome` tool；`work_item_get` 保持为原始机器记录查询。该 tool 返回稳定的
`structuredContent.outcome` 和可见的本地化 `humanHandoff`。Contract 原文保持不变，不推断人工决定。

发布 adopter 和 upgrade harness 使用带类型的 isolation manifest。每个 manifest 记录相对路径、entry type、
mode/size/mtime metadata，以及普通文件或 symlink target 的 SHA-256 digest。HOME 与 XDG_CONFIG_HOME 是禁止写入
的 root；TMPDIR 与 CARGO_HOME 是明确分类的 Runtime 写入 root。receipt 绑定 before/after manifest digest 和
经过校验的临时 root 清理结果。

## 验收

- CLI 与 MCP 使用同一个 renderer，并显示状态标记、unknown、证据、结构化人工决定投影和下一步。
- 覆盖中文、英文、日文 MCP handoff；Contract 验收标准保持原文。
- manifest 回归覆盖文件内容、目录、symlink target、metadata 变化，以及清理后无残留 root。
- 公开 v0.2.7 adopter acceptance 通过 `isolation.json` schema 2、typed manifest、`cleanup.json` 和目录级
  `SHA256SUMS`。
- repository-local Agent 指令说明 handoff 和隔离边界，不修改全局 Agent 或 MCP 配置。

## 验证

```text
cargo test --locked -p cockpit-mcp --test rpc -- --test-threads=1
cargo test --locked -p cockpit-cli --test intelligence --test outcome_human_decision -- --test-threads=1
bash tests/release/isolation_manifest_test.sh
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
bash tests/docs/documentation_acceptance.sh
```

公开验收只使用下载的 v0.2.7 binary，不回退到源码或 workspace binary。严格 typed verification evidence、foreign
Runtime policy、历史 evidence projection 和外部不可变 audit retention 仍由后续独立任务负责。

## Outcome

状态：**本地实现完成；MCP、CLI、文档、manifest 和公开 adopter 验收聚焦检查均通过。**

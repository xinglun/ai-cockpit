---
author: AI Cockpit maintainers
title: "WI-115——v0.2.9 发布与能力面一致性"
description: "发布 v0.2.9，并关闭发布前发现的参考源命令、MCP 与发布文档一致性缺口。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-115-release-v0-2-9
capabilityClaims:
  - release_distribution
  - reference_parity
  - cli_commands
---

# WI-115——v0.2.9 发布与能力面一致性

## 目标

从已审查的 default branch 发布下一版不可变 Release，并让参考源 Agent
规则、CLI/MCP 能力清单和发布示例对未来 Work Item 与 adopter 保持真实一致。

## 范围

- v0.2.9 三语版本与发布分发文档、当前基线和 N-1 示例；
- 三语 MCP 与 CLI 命令清单，包括 `delegated_evidence_list`、`capability show`
  和 `diagnose`；
- feature 与 reference-parity 文档中 Runtime 生成且校验的 `humanHandoff` 边界；
- 防止命令清单和发布 target 示例再次漂移的文档验收检查；
- 只使用下载的公开 artifact 执行不可变 v0.2.9 发布、adopter 验收和
  v0.2.8→v0.2.9 N-1 验收。

## 范围外

Runtime 行为、Protocol schema、全局 Agent/MCP 配置、外部 Homebrew tap 写入、
重写历史 Release/evidence，以及第二技术栈 adopter。

## 对应缺口

参考源比对确认，本仓库已经继承核心操作规则：一个 Work Item/branch/worktree/PR、
显式 repository binding、fail-closed preflight 与 Outcome、当前 WI 内修复、不可变
Release 验收，以及不写全局 Agent/MCP。剩余四项是文档漂移：少列一个 MCP 工具、少列两个
CLI 入口、发布 target 示例含义不清，以及把 MCP 面向人的 projection 错写成完全由 Agent
层生成。

## 验收

1. 三语 capability 页面列出 `tools/list` 返回的十二个工具；CLI 参考列出
   `capability show` 和 `diagnose`。
2. 发布文档把 v0.2.9 标为当前版本，并使用 `x86_64-unknown-linux-gnu` 作为完整
   adopter 基线示例；其他 target 明确是额外覆盖。
3. feature 和 parity 页面说明 Runtime 校验 OutcomeV2 并生成 `humanHandoff`；
   Agent/对话层只负责选择和展示，不能把 presentation 变成治理授权。
4. 文档验收、版本一致性、发布 policy、Rust 质量、conformance 和 adopter harness 均通过。
5. 公开 v0.2.9 artifact 通过 adopter 与 N-1 验收，证明 repository/runtime 隔离、
   cleanup、evidence reuse，且 `first-adopter-smoke = not_ready`。
6. Work Item 完成已安装 Runtime lifecycle，并输出面向人的 Outcome，包含 🟢/🟡/🔴、
   unknown、evidence、decision 和下一步。

## 继承边界

未来 Work Item 继承当前 `AGENTS.md`、`.ai/README.md` 和
`docs/reference/agent-workflow.*`。这些页面是 repository-local 操作权威；本记录是
发布 evidence，不替代使用路线。

## Release truth

已有 v0.2.8 Release 及其修复前/失败 receipt 保持不可变。发布后 adopter 失败只记录为
失败 evidence，不会重写已发布 Release truth。


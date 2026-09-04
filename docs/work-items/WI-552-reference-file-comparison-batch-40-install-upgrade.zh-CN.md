---
author: AI Cockpit maintainers
title: "WI-552——安装与升级参考源比较批次 40"
description: "逐个比较 17 个固定安装/升级路径，并收紧 Runtime capability discovery。"
audience: [maintainer, reviewer]
status: current
authority: canonical
workItemId: WI-552-reference-file-comparison-batch-40-install-upgrade
lastVerifiedBy: WI-552-reference-file-comparison-batch-40-install-upgrade
---

# WI-552——安装与升级参考源比较批次 40

## 目标

逐个比较固定参考源的安装/升级路径，在不复制 Python 实现、源 JSON wire、provider registry 或仓库内 installer state 的前提下，将可移植治理责任保留在共享 Rust Runtime。

## 范围与结果

本批覆盖 `tests/conformance/reference_file_inventory.json` 中记录的 17 个路径，包括安装事实、计划/状态/向导、仓库检测/证据/归属/事务、版本解析、升级应用/冲突/提案及 Python launcher。所有路径都明确分类为 `implemented-different-by-design` 或 `reference-only`，没有新增 `migrate-gap`。

Runtime 现在通过一个 Protocol-owned capability registry 生成 `.ai/agent-interface.json`。`attach` 暴露完整命令能力供 discovery；ready、授权、evidence 和生命周期门禁仍按请求绑定 repository。Agent 应先读取 manifest，再查询 CLI/MCP schema；列出的 capability 不是权限。

## 不声明

Runtime 安装在外部完成并由多个仓库共享。`attach` 只创建最小治理脚手架。源 installer catalog、Python launcher、provider policy、全局 Agent/MCP 配置和源 wire JSON 不会被对象工程继承。

## 验证

- Rust attach 回归测试完整 capability registry 与幂等 manifest bytes。
- inventory 与 shell conformance 检查覆盖 17 个源路径，并拒绝本批 deferred/migrate-gap。
- 三语 capability/configuration/reference/parity 文档说明 capability discovery 及 `--help`/MCP `tools/list` 查询方法。

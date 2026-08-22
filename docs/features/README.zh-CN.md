---
author: AI Cockpit maintainers
title: "功能"
description: "按目标查找 AI Cockpit 当前能力及边界。"
audience:
  - adopter
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - capability_index
---

# 功能

完整的面向用户功能索引见[功能与边界](../capabilities.zh-CN.md)。主要路径包括：

- attach 并观察 repository；
- 创建治理骨架，但不擅自生成人类决定；
- 使用有界验证和 evidence reuse 执行 Work Item 生命周期；
- 显式连接 Agent 或 repository-bound MCP service；
- 查看 Outcome、knowledge、status、diagnosis 和 recovery 信号。

AI Cockpit 是 Repository Governance Layer，不是 Agent Runtime、identity provider、security sandbox、
workflow scheduler 或外部 audit system。MCP 返回 repository-bound structured data；Agent 或对话层负责
面向人的 projection，并必须保留 unknown 与决定边界。为了获得一致的面向人 handoff，应调用
repository-bound `work_item_outcome`；它使用与 CLI 相同的 renderer。发布验收还会记录带类型的隔离
manifest 和 digest，只有 TMPDIR 与 CARGO_HOME 被分类为允许 Runtime 写入的 root。

[快速开始](../getting-started/README.zh-CN.md) | [运维](../operations/README.zh-CN.md) |
[参考](../reference/README.zh-CN.md) | [English](README.md) | [日本語](README.ja.md)

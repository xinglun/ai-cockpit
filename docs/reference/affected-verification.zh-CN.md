---
author: AI Cockpit maintainers
title: 受影响 Verification 与依赖置信度
description: 说明依赖知识 complete、partial、unknown 时的保守 Verification 计划。
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-142-affected-verification
---

# 受影响 Verification 与依赖置信度

Verification graph 将 `DependencyConfidence` 与 Policy truth 分开记录：

- `complete` 计算变更节点及已知下游依赖；
- `partial` 保留确定的受影响集合，将这些节点升级到更强的候选 tier，并暴露
  `dependency_graph_partial`；
- `unknown` 保守地包含 graph 的全部节点，并暴露 `dependency_graph_unknown`。

未知或不安全的节点引用必须 fail-closed。partial 不能当作 complete，但在已知
受影响边界足够时，也不强制所有节点无条件跑最高 tier。该 projection 只减少执行
成本，不能削弱 Policy tier、protected gate、authority 或 evidence requirement。

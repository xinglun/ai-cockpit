---
author: AI Cockpit 维护者
title: “Work Item 编写指南”
description: “编写可评审、以证据为边界的 Work Item 的实用指南。”
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-390-reference-style-guide
capabilityClaims:
  - work_item_style_guidance
---

# Work Item 编写指南

[English](work-item-style-guide.md) · [日本語](work-item-style-guide.ja.md)

本指南说明如何编写便于人评审、且可由已安装 Rust Runtime 验证的 Work Item。
它不是第二套 Contract schema。Contract 仍然是由人拥有的 intent、authority、scope、
acceptance 和 required evidence 的来源。

## 先说明结果

先描述完成后应该成立的结果，再描述实现方式。如果没有提供问题背景或用户收益，
应明确写出未知；不要从文件名、检测到的技术栈或 Agent 文字推断动机、影响、批准或完成。

有意识地使用当前 Contract 字段：

- `intent` 和 `goal` 表达由人拥有的目的与期望结果。
- 结构化 intent 可以记录 `businessGoal`、`userGoal`、`problem`、`constraints`、
  `nonGoals` 和 `rationale`；这些字段均可选，未由负责人提供时保持 unknown。
- `intentAlignment` 是实现后的可选 Summary 投影，用于记录问题、约束、非目标和理由是否
  实际得到处理；它不会改写原始 intent。

## 先定义问题和边界

只有在背景已知时才说明 Work Item 为什么存在。在编辑前声明相对于仓库的 `scope` 和
`outOfScope`。scope 是授权边界，不是事后罗列 changed files。明确非目标，使评审能够发现
意外扩张。

## 让验收可观察

验收标准必须能由人或声明的验证命令检查。优先使用“Contract validator 通过”或“文档中的
路由链接可解析”这样的表述，不要使用“看起来不错”这样的主观表述。编号为 `A<n>:` 的标准
可以绑定 Summary evidence；未编号标准继续作为可读的原语言声明。Runtime 不会擅自创建标准
或 evidence 映射。

## 治理决定由人拥有

`authority`、批准、风险接受，以及在出现 unknown 后是否继续，都属于负责的人或明确委托的
provider。Runtime 验证结构、身份、新鲜度和证据，但不会把缺失字段变成许可。黄色或红色
preflight 是评审边界，不是编辑或 finish 的授权。

## 使用足够小的流程

使用现有生命周期和验证能力。只有在确实保留评审或审计价值时，才增加字段、gate 或批准步骤。
按仓库选择相称的 Light/Standard/Strict profile；更高的 Verification Tier 不等于更高的
Evidence Assurance。

## 记录可执行的验证

声明本仓库可运行的检查，并使用已安装 Runtime 和显式 `--repo` 执行。Verification receipt
绑定 Work Item、仓库快照和 Runtime identity。声明本身不是证据，仅仅存在的路径也不是通过的检查。

## 添加概念前先扩展现有概念

在引入新概念前，先检查当前 Contract、Summary、scenario、evidence、decision 和 policy 字段。
先记录评审模型，只有需要确定性机器检查时才添加 schema。保持源语言和治理 bytes 不变；展示层
本地化不得改变其含义。

## 对象工程继承

对象工程通过仓库自己的 `.ai/` 和 Agent adapter 继承同样的面向读者规则，而共享 Runtime 仍在
工程外部。每个 `--repo` 的仓库 identity、Contract、evidence 和 knowledge 都相互隔离。本页
不复制参考源安装器的命令或 Runtime 实现，只在 Rust-native 接口中保留适用的治理语义。

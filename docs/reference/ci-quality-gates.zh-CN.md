---
author: AI Cockpit maintainers
title: CI Contract 感知质量门
description: 动态 CI 路由及其 Rust 原生 Contract 质量门。
audience:
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-291-ci-contract-aware-gates
---

# CI Contract 感知质量门

AI Cockpit 使用资源感知的质量路由。路由是动态的，并不意味着每次变更
都无条件执行最昂贵的检查：

- `light`：文档-only 变更的聚焦检查；
- `standard`：增加源码、测试和 workspace 检查；
- `strict`：治理、workflow、release-owned、高风险和未知表面必须使用。

阶段下限和风险升级可以提高 profile。请求的 profile 只能升级自动结果，
不能降低它。规范 manifest 仍然定义命令清单；route receipt 绑定 manifest、
Git base/head、变更路径、Contract 路径/摘要和有序 gate ID。

## Rust 权威与 Python shadow

对于存在 active Contract 的 standard/strict Pull Request，CI 在执行仓库命令
前运行只读 Rust gate：

```text
Python 路由/manifest 计划
        ↓
Rust Contract gate（权威，不写入 .ai）
        ↓
Python gate runner 与 Cargo/static checks（shadow 对照）
```

Rust gate 校验 regular Contract 文件、repository identity、base revision、
当前 snapshot、typed Contract 不变量、按 policy 绑定的
intent/scenario/operation/stage 路由，以及当前 Agent-Risk/preflight 投影。
它输出带稳定 receipt digest 的 `repository_contract_quality_gate` JSON，包含
decision state、verification tier 和 evidence assurance。黄色或红色结果以
非零退出，不能授权执行仓库命令。

在收敛阶段保留 Python 路由和 runner。只有 hosted shadow 对照证明语义一致后，
后续批次才可以删除重复 policy。此 gate 不实现参考源完整 workflow 矩阵、依赖
planner 或 release-preflight 顺序。

质量 workflow 运行于 Pull Request 和 `main` push；feature branch push 只由 Pull Request
workflow 覆盖，避免同一提交产生两套互相竞争的质量结论。当 push 事件包含一个 active
Contract 时，route 使用 Contract 记录的 base revision，
而不是 `github.event.before`；这样 push 检查与同一 Work Item/PR 的 base 保持一致，
不会产生重复的伪失败，而 Pull Request 事件仍然是 review authority。

## 以源代码为中心的快照身份

仓库快照身份以源代码为中心并绑定 Repository Context：绑定已跟踪的源代码树和非 `.ai` 的工作树事实，
排除 Git `HEAD`、绝对工作树路径以及仅治理用的 `.ai/` 提交。这样，验证成功后
正常提交 Contract、Summary 和 Outcome 记录不会使证据失效，同时源代码变更仍
不能复用过期证据。

## Evidence 与发布边界

CI gate 是针对 reviewed change 的源码构建检查。它记录 Runtime identity 供诊断，
但不是公开 Release artifact。Release 和 adopter 验收仍必须使用不可变下载 tag、
archive/binary 校验、SBOM/provenance 及发布验收 harness。
CI gate 永远不写入 `.ai/` 的 Contract、Summary、checkpoint、verification 或
decision；这些可变记录仍只由 lifecycle 命令产生。

对象工程也遵循同一边界：共享 Runtime 在工程外部，每次请求显式携带 `--repo`，
仓库 Evidence 保持隔离。

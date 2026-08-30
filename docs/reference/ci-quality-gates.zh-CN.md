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
lastVerifiedBy: WI-423-ci-convergence
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

## 运行收敛与过渡边界

Pull Request 运行共享 workflow/PR 并发组，并且只对 Pull Request 事件启用
`cancel-in-progress`。因此同一 PR 的新提交会取代旧运行；`main` push 和不可变的
release workflow 不会被这条策略取消。动态 route 先在一个轻量 job 中规划；文档-only
的 `light` route 会跳过 Windows 与 V1 oracle job，而 `standard`、`strict` route
仍然运行它们。这是成本选择，不会降低所选 profile 的必需检查。

选择 gate 之前，route 会在 active Work Item Summary 存在时检查它。`checkpointed` 或
`finish_ready` 必须恰好有一个 checkpoint，`finish_ready` 还必须有绿色 preflight。
失败、过期、格式错误或不可能的过渡标记会在 hosted repository gate 启动前停止。
失败使用稳定代码（例如 `lifecycle_transition_invalid` 或
`lifecycle_transition_stale`）和有界 remediation；route 绝不会把未知状态变成许可。

gate runner 会捕获命令输出，不再把每个 fixture 预期的负向诊断逐条重放。失败报告包含
按根因去重的 `failureRoots`（根因代码、受影响 gate ID 和 remediation）；原始命令输出
不会被计为第二个失败。通过的 repository-gate receipt 保持原有 schema，因此仍可作为
post-finalize evidence。

对象工程通过自己的 `.ai/` 与 Contract 继承相同的 route 和过渡边界。共享 Runtime 与
policy manifest 位于工程外部；Work Item 状态、Evidence 和失败回执保持仓库本地隔离，
不会与本项目共享。

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

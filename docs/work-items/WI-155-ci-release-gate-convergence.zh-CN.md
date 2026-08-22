---
author: AI Cockpit maintainers
title: "WI-155——CI/release gate 对齐"
description: "保持 release 测试确定性，并定义 Phase 1 Runtime shadow 为 execution smoke。"
audience:
  - adopter
  - contributor
  - maintainer
status: implemented
authority: canonical
lastVerifiedBy: WI-155-ci-release-gate-convergence
workItemId: WI-155-ci-release-gate-convergence
---

# WI-155——CI/release gate 对齐

WI-155 让 release source-quality gate 与 CI 的确定性逐 package Cargo 测试策略一致。每个 package 都使用
`--test-threads=1` 执行；单个 test binary 内仍保留 verifier 自身 worker 上限覆盖的并行能力。

Runtime shadow 被文档和静态检查明确为 Phase 1 **execution smoke（执行冒烟）**：它校验不可变的公开 binary 能执行一次绑定
repository 的 verification command。其 receipt 明确不覆盖 policy route/planner、affected graph 完整性、跨 Work Item 的 physical
execution，以及每个 Work Item 独立的 evidence receipt。这一边界不删除也不替代既有 Cargo 和 release gate。

证据：`.ai/evidence/WI-155-ci-release-gate-convergence.verification.json`。
决定：`.ai/decisions/WI-155-ci-release-gate-convergence.close.json`。

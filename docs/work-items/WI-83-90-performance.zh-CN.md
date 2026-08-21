---
author: AI Cockpit maintainers
title: "WI-83–WI-90 性能与 Runtime 效率"
description: "带 identity 的性能证据、有界调度、Repository context 复用和非规范缓存。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
lastVerifiedBy: performance-focused-tests
capabilityClaims:
  - performance_baseline
  - repository_context_isolation
  - resource_aware_verification
  - single_flight_execution
  - incremental_knowledge_cache
---

# WI-83–WI-90：性能与 Runtime 效率

本组能力优化重复的本地工作，但不改变治理权限。性能数据只有在携带 Runtime identity、
repository identity、采集时间、样本和明确预算时才属于 evidence。可移植的
`tests/performance/regression_gate.sh` 读取两份 JSON，并对缺失字段、identity 不一致、
格式错误和预算回归 fail-closed；它不会构建源码 fallback。

Repository 层提供 request-scoped `RepositoryExecutionContext`，捕获一个不可变 Git snapshot
并缓存派生 observation。`RuntimeSession` 可以保留显式绑定的 context，但没有全局 current
repository，bind、refresh、unbind 都必须带明确路径。因此 A/B 两个仓库的 identity 和 snapshot
保持隔离。

Git content identity 是针对声明相对路径文件的增量 Merkle cache。未变化的 metadata 可以复用
digest，内容变化只使对应 entry 失效，删除会移除 entry，绝对路径或越界路径 fail-closed。
Verification 保留依赖 DAG、受保护节点执行和 receipt binding，同时支持 resource weight 与显式
resource budget。weight 为零或超过预算时，在启动进程前拒绝。

`SingleFlightCoordinator` 只有在 repository、Work Item、command 和 Runtime identity 全部一致
时才合并并发请求。它是短暂优化；返回的 receipt 仍需通过普通 evidence store，coordinator 不提供
权限。Knowledge index 记录归档源输入 digest，源输入变化时重建；index 仍然只是非规范缓存。

重点证据：

```text
cargo test -p cockpit-verification --test execution --test graph
cargo test -p cockpit-git --test snapshot
cargo test -p cockpit-repository --test repository_context --test knowledge_cache
tests/performance/regression_gate_test.sh
```

本地性能仍然受平台影响；只有将采集 artifact 绑定到目标 Release 后，才可作为发布证据。

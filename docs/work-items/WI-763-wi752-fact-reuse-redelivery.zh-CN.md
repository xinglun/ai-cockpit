---
author: AI Cockpit maintainers
title: "WI-763——WI-752 P1 事实复用重新交付"
description: "从最新 main 重新验证 P1 事实复用，以原始测量和 fail-closed 决策记录优化结论。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-763-wi752-fact-reuse-redelivery
lastVerifiedBy: WI-763-wi752-fact-reuse-redelivery
---

# WI-763——WI-752 P1 事实复用重新交付

## Contract 与边界

本 Work Item 从最新远程 `main`（`a8fa804057cad8792f56b580f665046d2d3fc52d`）
重新进行仅测量交付。原因是不可变的 WI-752 PR #733 被 hosted quality 以
`docs_governance_integrity: invalid_premerge_finalize` 拒绝；该 PR、分支、
worktree 和证据保留为历史记录，不重写、不删除。

范围仅包括原始测量证据、本三语言报告和三个 reference-parity 投影。
生产 Runtime 行为、IncrementalMerkle、版本发布及历史 WI-752 资源不在范围内。

## 测量与结论

使用 `tests/performance/runtime_benchmark.sh` 做两轮同 Runtime 复测：一次预热，
对 `inspect`、`status`、`doctor`、`observe`、`work-item status` 和 `diagnose`
各取 8 个 warm 样本。首个样本保留为 Runtime 身份探针之后的首次独立 CLI 进程，
不宣称真正 cold cache。8 个样本不足以声明可靠 p95/p99，因此这些分位数保持不可用。

原始 fixture 为 `tests/performance/fixtures/WI-763-fact-reuse-measurement.json`。
两轮使用 Runtime `0.2.87`、相同 Runtime 摘要、仓库 revision、机器和不可用的文件系统
比较键。warm p50 变化约为：`status +0.4%`、`observe +2.1%`、`inspect +2.2%`、
`doctor -0.2%`、Work Item status `+0.5%`、diagnose `+0.3%`。同 Runtime 复测没有证明
候选优化或可靠收益，不接受生产优化，候选明确 decline。

Runtime 未提供的实际读取字节、哈希字节、Git 调用数、子进程、峰值内存和缓存失效事件
均标记为不可用，不填零。常驻 MCP 与并发请求不由此 portable harness 测量。

## 正确性与有效性

`cargo test -p cockpit-repository --test repository_context -- --nocapture` 通过 7/7。
覆盖单次 snapshot memoization、仓库隔离、显式 RuntimeSession 绑定、源文件或配置变化
后的 fail-closed 失效、观察阶段边界，以及治理使用已验证观察上下文。独立重复调用
`status` 与 `doctor` 的输出一致。

fresh verification evidence 之前，parity 行按
`进行中 → 验证关闭后已实现` 注册并绑定终态路径。关闭后由 Runtime promotion check
投影三个语言页面与 parity 行的终态状态。

## 终态证据

预期绑定为 archive
`.ai/work-items/archive/WI-763-wi752-fact-reuse-redelivery.contract.json`、
verification `.ai/evidence/WI-763-wi752-fact-reuse-redelivery.verification.json`、
finalization `.ai/decisions/WI-763-wi752-fact-reuse-redelivery.finalize.json`、
close `.ai/decisions/WI-763-wi752-fact-reuse-redelivery.close.json`。

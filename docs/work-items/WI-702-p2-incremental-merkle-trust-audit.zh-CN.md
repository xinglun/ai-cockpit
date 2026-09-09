---
author: AI Cockpit maintainers
title: "WI-702——P2 IncrementalMerkle 信任边界审计"
description: "在任何增量内容身份计算影响治理前，验证 metadata 复用边界。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-702-p2-incremental-merkle-trust-audit
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-712-wi702-finalization-recovery
recoveryDecision: .ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json
---

[English](WI-702-p2-incremental-merkle-trust-audit.md) · [日本語](WI-702-p2-incremental-merkle-trust-audit.ja.md)

# WI-702——P2 IncrementalMerkle 信任边界审计

## 意图与边界

本 Work Item 在任何未来增量计算被考虑前，先验证 `IncrementalMerkle` 的信任边界。
North Star 仍是 Calibrated Human-Agent Trust：仓库隔离、WI 隔离、授权边界、证据绑定和
恢复能力优先于缓存命中率指标。

当前仓库的调用图搜索只在 `crates/cockpit-git` 定义和 snapshot 测试中找到
`IncrementalMerkle` 及其 `refresh` 调用；没有生产治理、MCP、doctor、status 或 outcome
路径构造这个 helper。因此本 WI 不声称它影响受保护决定，也不声称生产性能收益。改动仅限
helper 和聚焦测试；其他 agent 的 tree、branch、PR 和 evidence 都在边界之外。

## 假设与已观察缺陷

改动前，当缓存的 size 和 mtime 相同时 helper 会跳过读取。这些 metadata 只是线索，不是
内容身份证明：等长替换并恢复 mtime 时可能复用旧摘要。如果该 helper 以后接入受保护决定，
这是不可接受的。

候选实现因此在每次 refresh 中重新读取并哈希每个声明的 regular file。它在读取前后检查
metadata；如果读取期间可检测到 size、时间戳、文件消失或文件类型变化，则返回明确的
`ChangedDuringRead` 错误。为保持兼容保留 public `files_reused` 字段，但现在固定为 0；
任何调用方都不能把 metadata 当作文件未变化的证明。

## 正确性证据

聚焦的 `cockpit-git` suite 覆盖：

- 等长内容修改并恢复原始 mtime；
- 替换、删除、重命名和文件到目录的类型变化；
- 路径逃逸拒绝；
- 读取期间 before/after metadata 发生可检测变化时返回 `ChangedDuringRead`。

基线 revision 的旧测试明确期望未变化文件被复用。候选测试现在证明未变化文件会重新读取，
且等长/恢复 mtime 场景会改变 Merkle root。这是信任修正，不是性能优化；如果未来有生产调用，
候选实现可能增加成本。

## 限制与治理状态

metadata guard 无法证明读取期间发生了同时改变字节并恢复所有可观察 metadata 的并发修改。
这仍是明确的有效性限制，不是缓存命中，也不是正确性结论。文件通知、持久索引、分层 Merkle
树、生产接入、release 行为和性能基准都不在本 WI 范围内。

本文不声称最终 PR、merge 或绿色治理结果。Runtime verification receipt、托管 review、archive、
close 和最终文档晋级仍是必需步骤。

## 合并后恢复边界

PR #700 已由仓库 owner review 并合并，merge commit 为
`bd00a7ce888c2d0dba012da21ba1616eeeab0014`，reviewed head 为
`4eedf23ddf1e4a0491fb978127d61d852e6a5a1f`。本 Work Item 的不可变 pre-merge
finalization root 绑定 `86f8535f`；其间的 range 还修改了 pending parity registry，
因此已安装 Runtime 正确拒绝把该 range 当作 append-only finalization transition。

WI-702 的 archive、verification、Outcome、Events、Contract 和 finalization bytes
继续作为历史证据保留。append-only recovery decision 是
`.ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json`；WI-712
负责新的合并后 finalization、parity 注册和准确 cleanup 边界。本恢复保持
Calibrated Human-Agent Trust North Star，不声称性能收益。

---
author: AI Cockpit maintainers
title: "WI-719——P1 大文件流式哈希实验"
description: "在不弱化证据或 fail-closed 治理的前提下测量大文件流式哈希；只有收益成立才评估保留。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-719-p1-streaming-hash
status: measured_declined
authority: human:repository-owner
lastVerifiedBy: WI-719-p1-streaming-hash
terminalArchive: .ai/work-items/archive/WI-719-p1-streaming-hash.contract.json
terminalVerification: .ai/evidence/WI-719-p1-streaming-hash.verification.json
terminalFinalization: .ai/decisions/WI-719-p1-streaming-hash.finalize.json
terminalDecision: .ai/decisions/WI-719-p1-streaming-hash.close.json
---

[English](WI-719-p1-streaming-hash.md) · [日本語](WI-719-p1-streaming-hash.ja.md)

# WI-719——P1 大文件流式哈希实验

## 结论

流式哈希候选已完成测量并拒绝保留，候选代码已回滚，生产行为未改变。North Star 仍为 Calibrated Human-Agent Trust。这是基于证据的拒绝，不表示候选实现被判定为不安全。

## 假设与边界

候选把变更文件的整体 `fs::read` 改为分块哈希，并限制文本捕获上限；完整读取成功后才提交摘要，保留原始摘要字节语义、`MAX_CHANGE_TEXT_BYTES` 边界及 fail-closed 读取失败行为。实验仅限 `crates/cockpit-git`，不改变缓存、IncrementalMerkle、授权、证据绑定、恢复、Outcome 或发布验收。

## 配对测量

baseline 与 candidate 都从 `caa6ddffcc1847c9a1161e1d8aa414f1acabd11e` 构建，Runtime 身份分别绑定；仓库快照、macOS arm64/10 CPU、10,331 个 tracked 文件、80,274,774 个 tracked 字节、621 个历史 WI 和 1 个 16 MiB 普通路径变更文件相同。每次保留原始顺序、首次测量、1 次 OS 缓存预热、20 个 warm 样本和 nearest-rank 分位数；20 个样本不足以提供 p99。

| 命令 | baseline warm p50/p95 ms | candidate warm p50/p95 ms | candidate 变化 |
| --- | ---: | ---: | ---: |
| status | 2,895.183 / 3,346.259 | 2,965.996 / 3,043.153 | +2.445% / −9.057% |
| inspect | 133.806 / 143.756 | 134.867 / 139.398 | +0.793% / −3.036% |
| doctor | 52.129 / 54.041 | 52.776 / 54.804 | +1.241% / +1.412% |
| observe | 188.036 / 204.221 | 186.403 / 207.658 | −0.869% / +1.684% |

Contract 要求 `status` 的 p50 和 p95 均至少提升 5%。候选未达到 p50，预算门禁报告 `budget_exceeded:status:2965.996>2739.035`。本机也无法提供可信文件系统类型或比较键，因此 comparator 正确 fail closed。Runtime 内部读取字节、哈希字节、Git 调用、子进程和峰值内存均明确不可用，不填零。

原始证据保存在 `.ai/evidence/external/WI-719-p1-streaming-hash.baseline.large-file.json`、`.ai/evidence/external/WI-719-p1-streaming-hash.candidate.large-file.json`；绑定决策和门禁结果汇总在 `.ai/evidence/external/WI-719-p1-streaming-hash.experiment-summary.json`。

## 正确性与治理

回滚前，候选的大文件捕获边界和原始摘要字节 focused tests 通过。去除 Runtime/binary 身份字段后，source baseline 与 candidate 的 `inspect`、`status`、`doctor`、`observe` 输出、错误和退出码一致。最终树保留并通过现有 repository 与 IncrementalMerkle 测试；不接受生产优化，也不宣称性能收益。

## 后续

后续性能 WI 应优先处理实测的大历史 `status` 瓶颈；除非隔离测量能证明可信的资源或延迟收益，不再推进大文件流式哈希。resident MCP、轮询、并行读取、协调器、缓存、进程内 Git 和 PGO 均不在本 WI 范围内。

## 验证限制

documentation acceptance 和 Work Item status consistency 已通过。仓库级
`promote_closed_work_item.py --check-all` fail closed，因为并行 WI-717 的文档投影缺少要求的普通 Markdown 文件。该独立 WI-717 缺陷记录在
`.ai/evidence/external/WI-719-p1-streaming-hash.validation-limitation.json`，本 WI 未修改它。

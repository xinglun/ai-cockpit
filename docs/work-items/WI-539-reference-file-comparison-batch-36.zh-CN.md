---
author: AI Cockpit maintainers
title: "WI-539——源治理检查器逐文件比较批次 36"
description: "逐个比较固定的十个参考治理检查器，记录 Rust 原生实现或外部边界。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-539-reference-file-comparison-batch-36
lastVerifiedBy: WI-539-reference-file-comparison-batch-36
terminalArchive: .ai/work-items/archive/WI-539-reference-file-comparison-batch-36.contract.json
terminalVerification: .ai/evidence/WI-539-reference-file-comparison-batch-36.verification.json
terminalFinalization: .ai/decisions/WI-539-reference-file-comparison-batch-36.finalize.json
terminalDecision: .ai/decisions/WI-539-reference-file-comparison-batch-36.close.json
---

# WI-539——源治理检查器逐文件比较批次 36

## 目标

在 pinned source commit `fde3380f81fea5fd2e288f7a8849f737dc074060` 上逐个阅读下一组 10 个维护中的参考检查器脚本，并为每个 current path 记录有证据的语义分类。这是 parity 与对象工程继承边界审查，不是把 Python、Make、YAML 或源 JSON wire 格式复制到 shared Rust Runtime。

## 文件级结果

| 参考路径 | 决定 | Rust 边界 |
| --- | --- | --- |
| `scripts/ai_check_guidelines.py` | `implemented-different-by-design` | typed Contract guidelines 仍由人维护；通过编号 acceptance/evidence 绑定完成性，不推断无类型的 `guidelinesCompliance` 声明。 |
| `scripts/ai_check_pr.py` | `implemented-different-by-design` | archive、recovery、scope、evidence 检查分布在 typed lifecycle gate；PR 身份和 hosted checks 仍是 provider evidence。 |
| `scripts/ai_check_reference_impact.py` | `reference-only` | 静态 AST/文本影响扫描保留为源/provider 工具。Rust 的 operation-time scope 检查 fail-closed，但不推导调用方、外部消费者或监控。 |
| `scripts/ai_check_registry.py` | `implemented-different-by-design` | 版本化 gate manifest 与 typed receipt 提供确定性的注册、去重和 unavailable-gate 原因。 |
| `scripts/ai_check_review_policy.py` | `implemented-different-by-design` | Contract/preflight 与 provider PR review 承载 authority；不安装第二套 YAML policy 或仅报告性的 focus list。 |
| `scripts/ai_check_scope.py` | `implemented-different-by-design` | repository-relative scope/out-of-scope、依赖、并行边界和 snapshot 检查由 typed Runtime gate 承载。 |
| `scripts/ai_check_serial_order.py` | `implemented-different-by-design` | predecessor、合并 PR、关闭、精确资源清理和同步 base 要求由 lifecycle 与 ready-on-base 检查执行。 |
| `scripts/ai_check_status.py` | `implemented-different-by-design` | request-scoped typed status 与 human Outcome projection 取代 generated `current_status.md` 作为权威。 |
| `scripts/ai_check_status_consistency.py` | `implemented-different-by-design` | 只读 status 推导 active/archive ownership 并拒绝歧义；Runtime 没有静默修复 generated status 的权限。 |
| `scripts/ai_check_summary.py` | `implemented-different-by-design` | 严格 Contract、evidence、archive、Outcome 绑定覆盖可移植边界，但不宣称源 Summary JSON 兼容，也不擅自补全人工声明。 |

## 结论与对象工程继承

本批没有发现可移植实现遗漏。reference-impact scanner 明确登记为 `reference-only`，不是隐藏的 Runtime 缺口：调用方和外部消费者等静态事实必须由对象工程/provider 或人维护的 evidence 提供，未知影响仍 fail-closed。其余 9 项责任由 typed Protocol、repository lifecycle、gate manifest、status 和 Outcome 边界承载。

每个 attach 的对象工程继承一份 shared Runtime、显式 `--repo` 绑定、隔离的 Contract/evidence/knowledge、fail-closed lifecycle 和人类 Outcome 展示。不会继承源检查器、provider policy 值或技术栈命令；源与目标的 JSON wire shape 保持独立。

## 验收

- inventory 在固定 source commit 上准确登记这 10 个 current path，且每项有非空 reason、counterpart 或明确边界。
- 选定路径不再保持 `deferred-next-batch` 或 `migrate-gap`，retired 历史继续追加保留。
- 英文、中文、日文比较页与本 Work Item 页陈述相同决定和对象工程边界。
- inventory、文档、格式化、lint 和 workspace 验证在 Finish 前通过。

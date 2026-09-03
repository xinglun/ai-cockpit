---
author: AI Cockpit maintainers
title: "WI-521——参考 guard 与采用检查批次 35"
description: "逐个比较下一组有界参考脚本，记录 Rust 原生边界，不复制源工具。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
workItemId: WI-521-reference-file-comparison-batch-35
lastVerifiedBy: WI-521-reference-file-comparison-batch-35
terminalArchive: .ai/work-items/archive/WI-521-reference-file-comparison-batch-35.contract.json
terminalVerification: .ai/evidence/WI-521-reference-file-comparison-batch-35.verification.json
terminalFinalization: .ai/decisions/WI-521-reference-file-comparison-batch-35.finalize.3963d731bcacd6a4efd4660409749638c2dcc8fe4bcde3a0bf2e8216fa12e2ae.json
terminalDecision: .ai/decisions/WI-521-reference-file-comparison-batch-35.close.json
---

# WI-521——参考 guard 与采用检查批次 35

## 目标

在 pinned commit `fde3380f81fea5fd2e288f7a8849f737dc074060` 逐个读取下一组
参考文件，并为每个 current path 记录有证据的分类。目标是语义 parity 和明确的对象工程边界，不是 Python/Make 命令兼容。

## 文件级结果

| 参考路径 | 决定 |
| --- | --- |
| `scripts/ai_check_adoption_ready.py` | `reference-only`：源专属采用完整性检查由 Rust onboarding 与 status/doctor 事实以外部边界承载。 |
| `scripts/ai_check_archive_recovery.py` | `implemented-different-by-design`：append-only archive 与前序绑定 recovery 保护不可变归属。 |
| `scripts/ai_check_backtrack.py` | `implemented-different-by-design`：Rust 推导测试/coverage weakening 与 input-trust；源 report-only 删除警告仍是维护投影。 |
| `scripts/ai_check_budget_impact.py` | `implemented-different-by-design`：typed identity-bound 性能/成本预算仅作 advisory，不替代必需验证。 |
| `scripts/ai_check_capability_claims.py` | `reference-only`：源 lexical claim/matrix 校验不是 Runtime authority；Rust 能力事实是 observed 且 repository-bound。 |
| `scripts/ai_check_coverage_guard.py` | `implemented-different-by-design`：Rust 检测 weakening 并绑定声明的验证；源 association 报告仍是对象工程 policy。 |
| `scripts/ai_check_dependabot_intake.py` | `not-applicable`：bot 事件身份和自动合并属于 provider。 |
| `scripts/ai_check_diff_ownership.py` | `reference-only`：Rust lifecycle scope 与 archive ownership 是 authority；不复制源跨 Work Item preview。 |
| `scripts/ai_check_guard_calibration.py` | `implemented-different-by-design`：Rust 校验 Project Profile 与显式校准事实。 |
| `scripts/ai_check_guards.py` | `implemented-different-by-design`：typed Contract、authority、trust、lifecycle、isolation 边界替代源 YAML manifest。 |
| `tests/test_ai_check_archive_recovery.py` | `implemented-different-by-design`：Rust native archive/finalization 测试覆盖不可变归属边界。 |
| `tests/test_ai_check_budget_impact.py` | `implemented-different-by-design`：Rust native verification/performance 测试覆盖 typed budget 与 exact reuse。 |

已退休的 `tests/test_ai_check_backtrack.py` 不作为当前源文件处理；其历史记录仍保留在 append-only 台账中。

## 验收

- 每个选定 current path 均从 pinned 本地 checkout 读取，并登记在
  `tests/conformance/reference_file_inventory.json`。
- inventory 回归测试确保 12 条记录都有非空 reason、counterpart 或明确边界，且没有选定记录保持 deferred。
- 不复制源 Python、Make、YAML guard、provider 配置，也不修改对象工程。
- 三语比较页面报告相同计数和相同语义边界。

## 对象工程继承

每个 attach 的对象工程继承 shared Runtime、显式 `--repo` context、repository-local Contract/evidence/knowledge、fail-closed lifecycle 检查和人类 Outcome 展示；不会继承源技术栈命令、Dependabot 事件、CODEOWNERS/SECURITY 值、Python 报告或示例 policy 决定。adopter/provider 事实仍是显式 external evidence。

## 验证

Finish 前必须通过 machine inventory check 与文档/conformance gate。本 Work Item 不新增 Runtime 代码或治理决定；未来可移植扩展必须使用新的有界 Contract。

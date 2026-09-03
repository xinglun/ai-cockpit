---
author: AI Cockpit maintainers
title: "WI-543 — 参考源逐文件比较批次 37"
description: "安全的 conformance 台账检查与七个源检查器比较。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: canonical
workItemId: WI-543-reference-ledger-check-safety
lastVerifiedBy: WI-543-reference-ledger-check-safety
terminalArchive: .ai/work-items/archive/WI-543-reference-ledger-check-safety.contract.json
terminalVerification: .ai/evidence/WI-543-reference-ledger-check-safety.verification.json
terminalFinalization: .ai/decisions/WI-543-reference-ledger-check-safety.finalize.json
terminalDecision: .ai/decisions/WI-543-reference-ledger-check-safety.close.json
---

# WI-543 — 参考源逐文件比较批次 37

## 目标

在 pinned source commit `fde3380f81fea5fd2e288f7a8849f737dc074060` 逐个比较下一个七个维护中的参考检查器模块，同时让清单校验器的 `--check` 模式保持只读。参考源是规格与行为语料，不复制 Python、Make、YAML、provider 或源 JSON wire 实现到 Rust Runtime。

## 文件级结果

| 参考路径 | 分类 | Rust 边界 |
| --- | --- | --- |
| `scripts/ai_check_task_outcome.py` | `implemented-different-by-design` | typed OutcomeV2/TaskOutcomeReport、追加式事件、三语 human handoff 与 archive 绑定覆盖可移植边界；不复制源报告 wire 与词法策略。 |
| `scripts/ai_check_test_weakening.py` | `implemented-different-by-design` | 基于 snapshot 的 Rust weakening signal 与 fail-closed unknown 覆盖可移植边界；源阈值和维护报告格式仍是源/provider policy。 |
| `scripts/ai_classify_operation_impact.py` | `implemented-different-by-design` | operation-time policy 与 scope 校验提供显式影响事实，不推断意图，也不导入源报告格式。 |
| `scripts/ai_close_work_item.py` | `implemented-different-by-design` | typed lifecycle/finalization/ready-on-base gate 执行收尾；provider PR 操作和源 runner 编排仍是外部责任。 |
| `scripts/ai_common.py` | `implemented-different-by-design` | JSON/Git/scope/redaction 分散在 typed Core、Protocol、repository 和 conformance 服务中，不复制单体 helper。 |
| `scripts/ai_critical_domain_guards.py` | `implemented-different-by-design` | typed operation、authority、prompt injection 与 evidence forgery 控制保持 fail-closed，不把词法分类提升为 authority。 |
| `scripts/ai_dependabot_intake.py` | `not-applicable` | Dependabot 事件身份和 bot branch 接入属于 provider；仍支持通用 delegated evidence 与 source binding。 |

## 台账安全

`reference_file_inventory.py --check` 严格只读。它会在加载或写入清单前拒绝 generation、rebaseline 和 apply 选项，避免误组合命令把追加式 retired history 替换成新生成投影。回归脚本同时检查拒绝行为和清单字节不变。

历史与 retired 记录按不可变记录验证；只有当前 pinned path 集合可以接受新的批次决定。这样源文件重命名、删除或 rebaseline 不会重新打开已完成比较。

## 对象工程继承

每个 attach 的对象工程都继承同一份 shared Runtime、显式 `--repo` 上下文、隔离的 Contract/evidence/knowledge、fail-closed 生命周期和 human Outcome handoff；不会继承源检查器、Dependabot/provider 事件、源 policy 值或源 JSON wire 格式。

## 验证

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --locked --workspace --all-targets --all-features -- --test-threads=1`

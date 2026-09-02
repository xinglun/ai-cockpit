---
author: AI Cockpit maintainers
title: "WI-504——参考文档第 29 批"
description: "逐个重读 5 个发生变化的本地参考文档，并在有证据时修复 Rust 读者入口遗漏，不复制源实现。"
audience:
  - maintainer
  - reviewer
workItemId: WI-504-reference-file-comparison-batch-29
status: implemented
authority: human-authorized
lastVerifiedBy: WI-504-reference-file-comparison-batch-29
terminalArchive: .ai/work-items/archive/WI-504-reference-file-comparison-batch-29.contract.json
terminalVerification: .ai/evidence/WI-504-reference-file-comparison-batch-29.verification.json
terminalFinalization: .ai/decisions/WI-504-reference-file-comparison-batch-29.finalize.json
terminalDecision: .ai/decisions/WI-504-reference-file-comparison-batch-29.close.json
---

# WI-504——参考文档第 29 批

[English](WI-504-reference-file-comparison-batch-29.md) · [日本語](WI-504-reference-file-comparison-batch-29.ja.md)

## 目标

在固定的本地参考源 checkout 上逐个比较下一批 5 个发生变化的文件。通过
Rust 原生读者路线保留可移植治理语义，并在证据确认存在具体导航遗漏时修复。
本 Work Item 不复制源 Python、Make、provider 命令、源 receipt，也不修改对象/采用方工程。

## 范围与文件决定

参考源提交为 `fde3380f81fea5fd2e288f7a8849f737dc074060`。每个路径都有明确台账决定：

| 参考路径 | 决定 | Rust 边界 |
| --- | --- | --- |
| `docs/reference/repository-workflow.ja.md` | implemented-different-by-design | Rust 日文 workflow 已使用本地化 Runtime 展示，不需要被删除的 `REPORT_LANGUAGE` 参数，并保留显式 repository-scoped 生命周期与清理。 |
| `docs/reference/troubleshooting.md` | implemented-different-by-design | Rust 三语 troubleshooting 路线保留通用停止/恢复与证据保留规则；provider handoff 记录仍是外部边界。 |
| `docs/reference/verification-evidence-reuse.md` | implemented-different-by-design | 源端 no-change 决定只针对其 Python/Make 提案；Rust 独立授权的 reuse 仍绑定 identity 且 fail-closed。 |
| `docs/reference/work-item-lifecycle-closure.md` | implemented-different-by-design | Rust 原生 closure、精确清理和 recovery 路线保留可移植边界；源 hosted-governance/Make recovery 细节不是 Runtime 命令。 |
| `docs/upgrade.md` | implemented-different-by-design | 新增最小根级兼容指针，恢复到规范 Rust 三语 upgrade reference 的读者路线。 |

## 验收

- 在固定参考提交上逐个重读 5 个路径，并在台账中记录非 deferred、有证据的决定，且每条都有对应物和原因。
- 根级 `docs/upgrade.md` 存在并指向规范 upgrade reference，不复制源实现或源声明。
- 三语比对/parity 文档记录相同的 5 个决定；当前计数一致且 `migrate-gap` 保持为 0。
- 不修改源实现、provider 配置、全局 Agent/MCP 设置或对象/采用方工程。
- 声明的 conformance、文档、Runtime 验证、reviewed PR、合并、close 与精确清理检查全部通过。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

台账表达的是语义/文档 parity，不是源命令、JSON wire、provider 状态或发布声明兼容。
参考源通过 `AI_COCKPIT_REFERENCE_ROOT` 读取，本 Work Item 不修改它。

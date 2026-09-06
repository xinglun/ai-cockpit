---
author: AI Cockpit maintainers
title: "WI-617——参考 Outcome、事件与人类交接对等"
description: "逐文件比较维护中的参考源 Outcome、事件和人类交接测试，并记录 Rust 语义边界。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-617-reference-outcome-parity
status: implemented
authority: canonical
lastVerifiedBy: WI-617-reference-outcome-parity
terminalArchive: .ai/work-items/archive/WI-617-reference-outcome-parity.contract.json
terminalVerification: .ai/evidence/WI-617-reference-outcome-parity.verification.json
terminalFinalization: .ai/decisions/WI-617-reference-outcome-parity.finalize.json
terminalDecision: .ai/decisions/WI-617-reference-outcome-parity.close.json
---

# WI-617——参考 Outcome、事件与人类交接对等

## 意图

逐文件比较参考源中维护的 Outcome、事件日志、多语言、不支持声明、schema、生成器、渲染器和 validator 测试，并记录 Rust 对应实现与边界，防止发布后静默留下可移植遗漏。

## 边界

固定源为本地参考 checkout 的 `fde3380f81fea5fd2e288f7a8849f737dc074060`。本批是语义比较，不复制源文件、Python、Make、供应商输出或 JSON wire。Rust Runtime 负责仓库绑定的证据与人类 Outcome 事实；适配器可以提供供应商展示，但不能成为第二个权威。

## 逐文件决定

11 条源路径及其对应实现记录在[机器台账](../../tests/conformance/reference_file_inventory.json)，摘要见[参考比较台账](../reference/reference-file-comparison.zh-CN.md#wi-617参考-outcome事件与人类交接对等)。其中 10 条为 `implemented-different-by-design`；供应商/适配器 PR summary 投影为 `not-applicable`。Runtime 的 canonical surface 是类型化 OutcomeV2、Task Outcome report/event 和可见的人类交接。

对象工程继承 shared Runtime、显式 repository context、隔离 Contract/evidence/knowledge、fail-closed 证据和人类 Outcome 边界；不会继承源 Python 测试、Make target、供应商 PR 格式或 source wire。

## 验证

在本仓库使用显式 repository context 运行：

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cf361147ede1f37fb8929eba26a78abffbd943b2
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

Runtime verification evidence 与终态 Outcome 是完成依据。Contract acceptance 保留原语言；只有固定展示文字做本地化。

参见：[English](WI-617-reference-outcome-parity.md) · [日本語](WI-617-reference-outcome-parity.ja.md)。

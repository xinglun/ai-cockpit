---
author: AI Cockpit maintainers
title: "WI-619——参考源 adoption、企业、生命周期与信任测试对等"
description: "逐文件比较下一批维护中的参考源测试，并记录 Rust 语义对应。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-619-reference-script-semantic-batch
status: in_progress
authority: canonical
lastVerifiedBy: WI-619-reference-script-semantic-batch
---

# WI-619——参考源 adoption、企业、生命周期与信任测试对等

## 意图

逐一比较下一批 21 条维护中的参考源测试，记录有证据支持的 Rust 对应实现或明确的源工程/供应商边界。本批覆盖 adoption、锁定环境、企业证据、依赖投影、生命周期准备度、治理 guard、托管验证和输入信任。

## 边界

固定本地参考源提交为 `fde3380f81fea5fd2e288f7a8849f737dc074060`。这是语义对等，不复制 Python、Make、供应商配置、fixture 矩阵或源 JSON wire。可移植治理由共享 Rust Runtime 与仓库原生测试负责；供应商和对象工程保留自己的工具链与外部 assurance。

## 决定

21 条路径全部记录在[机器台账](../../tests/conformance/reference_file_inventory.json)中，分类为 `implemented-different-by-design`。逐文件表见[参考比较台账](../reference/reference-file-comparison.zh-CN.md#wi-619参考源-adoption企业生命周期与信任测试对等)。未发现 `migrate-gap`。对象工程继承共享 Runtime、显式 repository context、隔离 Contract/evidence/knowledge、动态验证、fail-closed 生命周期和人类 Outcome 边界；不继承源 fixture 栈或供应商命令。

## 验证

使用显式 repository context 运行：

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit 2536e4db399a7c20479e09693c8eab968c635ac9
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
cargo test --locked --workspace
```

Contract 的 acceptance 保留原语言，只有固定展示文字做本地化；不新增未经证据支持的收益或合规声明。

参见：[English](WI-619-reference-script-semantic-batch.md) · [日本語](WI-619-reference-script-semantic-batch.ja.md)。

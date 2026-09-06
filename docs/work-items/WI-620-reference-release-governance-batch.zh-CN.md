---
author: AI Cockpit maintainers
title: "WI-620：参考安装、发布、质量与生命周期对等"
description: "逐文件比较剩余源内容变化的参考测试，并记录 Rust 对应。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-620-reference-release-governance-batch
status: in-progress
authority: canonical
lastVerifiedBy: WI-620-reference-release-governance-batch
---

# WI-620：参考安装、发布、质量与生命周期对等

## 意图

逐一重新阅读剩余 22 个源内容已变化的参考测试文件，记录有证据支持的
Rust 对应实现或明确边界。目标是 Runtime 和对象工程的语义覆盖，不复制源
Python、Make、供应商配置或源 JSON wire。

## 边界与决定

固定本地参考提交为
`fde3380f81fea5fd2e288f7a8849f737dc074060`，目标比较提交为
`cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd`。其中 21 个路径为
`implemented-different-by-design`；`tests/test_install_plan.py` 为
`reference-only`，因为 Rust 使用不可变 Release 产物和显式 `attach --repo`，
不复制源交互式技术栈/provider 向导。本批没有 `migrate-gap`。

逐文件台账见
[reference-file-comparison.zh-CN.md](../reference/reference-file-comparison.zh-CN.md#wi-620参考安装发布质量与生命周期测试对比)，
机器记录见
[reference_file_inventory.json](../../tests/conformance/reference_file_inventory.json)。
22 条路径覆盖已安装 Runtime 对等、安装器、Make/CI、PR 聚合、项目治理、质量
架构与测量、参考影响、Release 分发/预检/状态/工作流、start/archive、供应链、
已发布 Release 投影、验证策略、Work Item intelligence/lifecycle 以及 workflows。

## 对象工程继承边界

对象工程继承 shared external Runtime、显式 repository context、隔离的
Contract/evidence/knowledge、动态验证、fail-closed 生命周期和可见的 human
Outcome 边界。不继承源 Python 测试、Make target、交互式技术栈安装器、供应商
policy 值、fixture 栈或源 wire 格式。这是语义对等，不是源实现或 wire 兼容。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --check --source-commit fde3380f81fea5fd2e288f7a8849f737dc074060 --target-commit cb8248fdf8ac8d965d8d8eb7b53760147bd13fcd
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
python3 tests/conformance/reference_inventory_docs_test.py
python3 tests/docs/reference_comparison_metadata_test.py
cargo test --locked --workspace
```

Contract acceptance 保留原语言；只可本地化展示外壳，不翻译或虚构治理事实和人工决定。

参见：[English](WI-620-reference-release-governance-batch.md) ·
[日本語](WI-620-reference-release-governance-batch.ja.md)。

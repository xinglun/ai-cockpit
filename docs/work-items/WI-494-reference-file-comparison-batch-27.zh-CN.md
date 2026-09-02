---
author: AI Cockpit maintainers
title: "WI-494——能力、理解度与 deprecated-assets 参考源重新比对"
description: "逐个重读 7 个发生变化的本地参考源记录，并保留明确的 Rust 原生边界。"
audience:
  - maintainer
  - reviewer
workItemId: WI-494-reference-file-comparison-batch-27
status: in_progress
authority: canonical
lastVerifiedBy: WI-494-reference-file-comparison-batch-27
---

# WI-494——能力、理解度与 deprecated-assets 参考源重新比对

## 目标

逐个重读此前判定为 `reference-only`、但源内容发生变化的 7 个本地参考源路径。为每个路径记录有证据的有界决定，不把源研究数据、Python/Make 实现或源端清理工具复制到 Rust 仓库。

## 范围与边界

7 个源路径为：

- `docs/reference/capability-truth-matrix.json`
- `docs/reference/comprehension-validation-responses/peter_01.en.json`
- `docs/reference/comprehension-validation-responses/tanaka_01.ja.json`
- `docs/reference/comprehension-validation-responses/xiaoli_01.zh-CN.json`
- `docs/reference/comprehension-validation-results.json`
- `docs/reference/comprehension-validation-results.md`
- `docs/reference/deprecated-assets-registry.json`

7 个路径全部保持 `reference-only`。能力矩阵是源工程自有的声明/freshness 投影；参与者回答和理解度报告是绑定修订版的研究证据；deprecated-assets 注册表是源工程专用的清理辅助记录。Rust 通过 typed request-scoped 能力视图、面向读者的 Outcome 文档、不可变 Work Item 历史和经审查的资源收尾保留适用边界。源字节不会成为 Runtime 授权或 adopter 证据。

清单应用和回归测试保留每个路径的此前分类与 `sourceChangedSincePrevious` provenance。三语比对与 parity 路线记录相同的不复制决定。

## 验收

- 逐个重读 7 个路径，并在清单中以 `reference-only` 记录，且每条都有 Rust 对应物与原因。
- 不把参与者、理解度、源能力声明或源清理注册表字节复制到 Runtime 或 adopter 状态。
- inventory 校验、conformance 回归、文档验收、parity 状态检查及仓库声明的 Runtime 验证全部通过。
- 通过 reviewed PR 生命周期和精确清理交付；不修改全局 Agent/MCP 配置或对象/采用方工程。

## 验证

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

参考源 checkout 由 `tests/conformance/reference-source.lock` 本地固定；不要求源实现或 JSON wire 兼容。

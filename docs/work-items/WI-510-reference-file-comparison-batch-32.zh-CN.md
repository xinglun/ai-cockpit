---
author: AI Cockpit maintainers
title: "WI-510——安装入口与向导 locale 边界"
description: "逐个比较 4 个参考源安装/本地化文件，不复制源安装器或向导实现。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-510-reference-file-comparison-batch-32
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-510-reference-file-comparison-batch-32
terminalArchive: .ai/work-items/archive/WI-510-reference-file-comparison-batch-32.contract.json
terminalVerification: .ai/evidence/WI-510-reference-file-comparison-batch-32.verification.json
terminalFinalization: .ai/decisions/WI-510-reference-file-comparison-batch-32.finalize.json
terminalDecision: .ai/decisions/WI-510-reference-file-comparison-batch-32.close.json
---

[English](WI-510-reference-file-comparison-batch-32.md) · [日本語](WI-510-reference-file-comparison-batch-32.ja.md)

## 目标

逐个阅读固定参考提交中的 `install.sh` 以及英文、日文、简体中文向导 locale 文件，为每个路径记录有证据的语义决定和 Rust 对应边界。本任务只做比较与边界定义，不复制源 Shell/Python 安装器、向导、locale 字节或源 JSON wire shape。

## 逐文件决定

| 固定路径与源 digest | 分类 | 目标边界 |
| --- | --- | --- |
| `install.sh`——`sha256:14f157f828e3ba8d1dd0886708b7eae223fe6d08` | implemented-different-by-design | Rust 不可变公开 Release、checksum/SBOM/provenance、显式 repository attach 与隔离 adopter 验收承载源选择、校验、清理、回滚和隔离语义。不加入源 Shell/Python 安装器，也不隐式写入目标工程。 |
| `locales/wizard/en.json`——`sha256:1b9bfc3535e507c8478b071b641d974cb031e59e` | reference-only | Rust 英文 Runtime 标签和 human Outcome 在安装、命令、Outcome 文档中说明；交互向导 prompt/session 控件属于宿主/Agent adapter UX。 |
| `locales/wizard/ja.json`——`sha256:8fab9ba89bd2bac5ccd51e8cb70dfea719435f5c` | reference-only | Rust 日文 Runtime 展示有文档说明；不提供第二套交互安装器，locale 文本不能授权 repository 修改。 |
| `locales/wizard/zh-CN.json`——`sha256:591e11709864edf2846bfe63aab246b1dafd6473` | reference-only | Rust 中文 Runtime 展示有文档说明；不复制源向导字节，locale 不能授权 repository 修改。 |

## 对象/采用方工程继承边界

每个对象或采用方工程在外部安装一份共享 Runtime，并通过显式 `--repo` 绑定自己的 repository context。它继承 repository-local 的 `attach`、Agent adapter、Contract、evidence、knowledge 与 human Outcome 边界，不继承源安装器实现、技术栈专属向导、源 locale JSON 或 provider 决定。Contract 事实保持作者语言，只有 Runtime 自有展示进行本地化。

## 验收标准

- 4 个固定路径都有源 digest、理由和对应清单。
- 安装器语义由 Rust Release/distribution 与 adopter 文档承载，不复制源代码。
- locale 保持 reference-only，同时明确 Runtime 多语言展示和 adapter 责任。
- 英文、中文、日文 inventory、comparison、parity 和本 Work Item 文档同步，且无 `migrate-gap`。
- conformance、文档和 workspace 验证通过，不修改对象工程、全局 Agent/MCP 配置或无关 Runtime 行为。

## 验证命令

Contract 中声明的检查为：

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

参考源 checkout 只用于本地比较，不向本仓库加入源安装器、locale 或其他源文件。

## 终态证据

front matter 中列出的生成 archive、verification、finalization 和 close receipt 是生命周期状态的权威来源。comparison 页面记录相同的 4 项决定和当前 inventory 计数；历史 evidence 不会被改写。

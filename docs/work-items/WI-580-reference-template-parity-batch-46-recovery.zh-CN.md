---
author: AI Cockpit maintainers
title: "WI-580——参考模板对等批次 46"
description: "逐个重读剩余 16 个参考模板路径，记录有边界的 Rust 语义决定，不复制源实现。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-580-reference-template-parity-batch-46-recovery
lastVerifiedBy: WI-580-reference-template-parity-batch-46-recovery
---


> This is an independent replacement delivery for the immutable WI-579 attempt. WI-579 registered the parity rows after archive, so its delivery remains an auditable failed attempt. This Work Item starts from the reviewed default branch and registers the parity rows before verification; it does not rewrite WI-579 history.

[English](WI-580-reference-template-parity-batch-46-recovery.md) · [日本語](WI-580-reference-template-parity-batch-46-recovery.ja.md)

# WI-580——参考模板对等批次 46

## 目标

逐个阅读固定本地参考源提交
`fde3380f81fea5fd2e288f7a8849f737dc074060` 中剩余的全部 `templates/**`
路径，为每个路径记录明确的 Rust/仓库原生语义对应，或有边界的
`reference-only` 决定。本批是语义对等，不是源实现、Make target、技术栈命令
或 JSON wire 的迁移。

## 逐文件决定

| 固定参考路径 | 分类 | Rust 对应 / 有边界决定 |
| --- | --- | --- |
| `templates/agents/AI_COCKPIT_RULES.md` | implemented-different-by-design | `AGENTS.md`、`.ai/README.md`、`.ai/glossary.md`、`crates/cockpit-agent/src/lib.rs` 和三语 Agent 工作流保留仓库绑定、Contract 优先审查、暂停规则、证据、Outcome 与精确清理；不复制模板 Markdown/Make 表面。 |
| `templates/glossary.md` | implemented-different-by-design | `.ai/glossary.md`、`docs/reference/commands.md` 与 `docs/reference/agent-workflow.md` 承载治理词汇；项目领域占位词由 adopter 自己定义，Runtime 不臆造。 |
| `templates/make/Makefile.ai` | implemented-different-by-design | Rust CLI/Repository/Verification 服务和受审查 gate manifest 承载生命周期、质量和证据责任；源 Make/Python target 名称与 shell 默认值仍是 adopter/provider 集成选择。 |
| `templates/stacks/android.mk` | reference-only | Gradle/Android 技术栈命令只是源模板便利默认值；adopter 自己声明工具链和 verification argv，shared Runtime 不推断或复制。 |
| `templates/stacks/csharp.mk` | reference-only | .NET 命令属于 adopter 委托检查，不随 Runtime 提供 C# preset。 |
| `templates/stacks/flutter.mk` | reference-only | Flutter/Dart 工具链默认值属于源/adopter 配置，不是 Runtime 治理。 |
| `templates/stacks/generic.mk` | reference-only | generic fail-closed 占位内容只是源模板 onboarding 辅助；Runtime 显示缺失检查，不制造命令。 |
| `templates/stacks/go.mk` | reference-only | Go 格式化、测试、lint 命令属于 adopter 委托检查，不是可移植 Runtime Contract。 |
| `templates/stacks/java.mk` | reference-only | Java/JAVA_HOME 与 Gradle/Maven 选择是技术栈/provider 事实；Runtime 不选择或安装 JDK。 |
| `templates/stacks/kotlin.mk` | reference-only | Kotlin/Gradle 默认值是源模板便利配置，位于 Core 之外。 |
| `templates/stacks/php.mk` | reference-only | PHP 格式化、测试、静态分析命令由 adopter 明确声明。 |
| `templates/stacks/python.mk` | reference-only | Python/Ruff/Pytest 是源模板开发工具；Rust Runtime 不安装或复制 Python 环境。 |
| `templates/stacks/ruby.mk` | reference-only | Ruby/Bundler/Rake 命令属于 adopter 委托验证。 |
| `templates/stacks/rust.mk` | reference-only | Cargo 命令可以是 adopter 选择，但不复制为 stack preset；Runtime 只使用仓库声明且 profile 授权的验证路线。 |
| `templates/stacks/swift.mk` | reference-only | Swift/SPM/Xcode 假设属于 adopter/平台；Runtime 不宣称 Xcode 或 CocoaPods 覆盖。 |
| `templates/stacks/typescript.mk` | reference-only | npm 格式化、测试、lint 默认值由 adopter 负责，shared Runtime 不推断。 |

## 边界与对象工程继承

三个 `implemented-different-by-design` 决定通过 shared external Runtime 和仓库本地
文档保留可移植治理责任。13 个 stack 文件明确为 `reference-only`，因为其命令、工具链
版本和平台假设不能安全地普遍化。每个 attach 的对象/adopter 工程继承 shared Runtime、
显式 `--repo` 上下文、隔离的 Contract/evidence/knowledge、动态验证边界和面向人的
Outcome handoff；不会继承源 Python、Make、stack preset、provider policy 值或 source wire。

## 验证

- `python3 tests/conformance/reference_file_inventory.py --manifest tests/conformance/reference_file_inventory.json --apply-wi579-batch`
- `bash tests/conformance/reference_file_inventory_test.sh`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `python3 tests/docs/reference_comparison_metadata_test.py`
- `bash tests/docs/documentation_acceptance.sh`
- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `git diff --check`


---
author: AI Cockpit maintainers
title: "WI-391——C# 适配示例"
description: "比较固定 C# 适配示例，不复制其安装器或旧 wire 格式。"
workItemId: WI-391-reference-csharp-adaptation
audience:
  - adopter
  - contributor
  - maintainer
authority: canonical
status: implemented
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
lastVerifiedBy: WI-391-reference-csharp-adaptation
terminalArchive: .ai/work-items/archive/WI-391-reference-csharp-adaptation.contract.json
terminalVerification: .ai/evidence/WI-391-reference-csharp-adaptation.verification.json
terminalFinalization: .ai/decisions/WI-391-reference-csharp-adaptation.finalize.daf3f48ceb9d6aa46efc7a12f2251b2013c5efbeb79a1ef6a96b38811edad407.json
terminalDecision: .ai/decisions/WI-391-reference-csharp-adaptation.close.json
---

# WI-391——C# 适配示例

[English](WI-391-reference-csharp-adaptation.md) · [日本語](WI-391-reference-csharp-adaptation.ja.md)

## 意图与边界

逐节比较固定的 `examples/csharp/README.md`，以 Rust 原生方式记录适用于 C#/.NET adopter 的语义。
目标是说明共享 Runtime 与 repository-local 责任，不把源安装脚本、Makefile、guard YAML、Python 编排或
旧 JSON 示例变成目标工程的要求。

## 范围

- 添加三语 C# 适配参考页及 reference index 链接。
- 在三语比较和 parity 台账中记录安装、质量门、Contract、coverage 与 guideline evidence 映射。
- 明确固定源提交及语义/非 wire 边界。

## 不在范围内

本 Work Item 不添加 .NET 工具、C# fixture、第二技术栈 adopter 验收、安装器实现、Makefile、源 guard
解析器、Python checks、provider 集成或新的 Contract wire schema。

## 验收标准

1. 源的四个章节（安装；质量门与 guard；Contract；`guidelinesCompliance`）均有 Rust 原生映射，或明确标记为外部/不适用。
2. 源 front matter 被视为描述性元数据，不是目标 authority 或能力声明；源安装变量/flags 也明确映射到不同的 Rust 安装边界。
3. 适配页说明一份不可变共享 Runtime、显式 `attach --repo`、repository-local `.ai/`、显式 Agent adapter
   设置以及由 adopter/provider 负责的 `dotnet` 检查。
4. 明确源 `contractVersion: 2`、`ai*` verification 名称、`Makefile.ai.stack` 和 `guidelinesCompliance`
   不是 Rust JSON-wire 要求，并使用当前 Contract/evidence/decision 边界。
5. 英文、简体中文、日文页面、index 链接、inventory 和 parity 行保持同步。
6. 使用新鲜 Runtime verification 通过文档与 conformance 检查。

## Evidence 边界

本 Work Item 是文档/语义 parity，不证明已经运行 C# adopter。未来 C# 验收必须使用不可变公开 Release、独立
repository context 及自己的 evidence/decision 链。

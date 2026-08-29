---
author: AI Cockpit maintainers
title: "C# 技术栈适配"
description: "面向 C# adopter 的 Rust 原生、repository-bound 映射。"
audience:
  - adopter
  - contributor
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-391-reference-csharp-adaptation
capabilityClaims:
  - csharp_adaptation_guidance
---

# C# 技术栈适配

[English](csharp-adaptation.md) · [日本語](csharp-adaptation.ja.md)

本文是固定参考源 `examples/csharp/README.md` 的 Rust 原生对应页，逐节比较并保留适用于 .NET
repository 的治理含义。它不是第二套 Contract schema、.NET 工具链保证，也不是第二技术栈
adopter 验收回执。

## 逐节比较

| 参考源章节 | 保留的含义 | Rust 原生边界 |
| --- | --- | --- |
| 源元数据 | 标题和 keywords 表明这是 C# 适配示例。 | 目标页使用 Rust 文档的 canonical 元数据；源 front matter 只是描述信息，不是能力或权限记录。 |
| 安装 | 源文件要求不可变 template tag/raw base，并通过安装器选择 C#、生成 adoption 文件。 | 在 repository 外安装一份不可变共享 Runtime，校验 archive 和 binary 的 SHA-256，再显式执行 `attach --repo <path>`。Runtime 不运行源安装器、不生成 Makefile，也不静默选择 provider。 |
| 质量门与 guard | 格式化、测试、警告构建以及生产/测试路径边界应显式声明。 | `dotnet format --verify-no-changes`、`dotnet test`、`dotnet build -warnaserror` 仍由 adopter 或 provider 负责。路径边界写入 Contract scope/outOfScope 或 repository policy；不要求源 YAML guard 文件。 |
| Contract 示例 | Work Item 声明 identity、mode、scope、指导、验证和 acceptance。 | 使用 `work-item new` 生成当前 Rust Contract，再填写人类拥有的 intent、scope、acceptance、authority 和 evidence 要求。源字段名不意味着 JSON-wire 兼容。 |
| `guidelinesCompliance` 示例 | Summary 应说明如何满足人类指导并引用 evidence。 | 指导保存在 Contract `guidelines`，通过编号 acceptance evidence、`intentAlignment` 或委托 evidence 绑定证明。不添加无类型合规声明，也不在没有 evidence 时标记为 true。 |

## 不复制源安装器的安装与 attach

安装属于 Runtime 边界，不是项目模板生成操作。选择不可变公开 Release，校验 archive 和 binary 的
SHA-256，在 repository 外安装一次 binary，然后为每个 repository 显式绑定：

```bash
repo=/path/to/csharp-repository
ai-cockpit --version
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

确认 profile 前先审阅检测到的项目事实。Agent adapter 安装是独立且显式的 repository 操作，不会修改
全局 Agent 或 MCP 配置。参见[安装 AI Cockpit](../getting-started/installation.zh-CN.md)、
[Adopter 配置](../getting-started/adopter-configuration.zh-CN.md)和[已安装 Runtime 生命周期](installed-lifecycle.zh-CN.md)。

参考源的 `AI_COCKPIT_TEMPLATE_REF`、`AI_COCKPIT_TEMPLATE_RAW_BASE`、通过 `curl` 下载的 `install.sh` 以及
`--stack csharp --update-makefile --create-adoption` 不复制到这里。它们描述源安装器流程；Rust 路线将
Runtime 分发、repository scaffold、项目 policy 和 Agent discovery 保持为可分别审查的边界。

## 由 adopter 负责的 C# 验证 evidence

以下是项目事实示例，不是 Runtime 自带命令：

```bash
dotnet format --verify-no-changes
dotnet test
dotnet build -warnaserror
```

人类拥有的 Contract 准备好后，用已安装 Runtime 绑定新鲜结果，例如：

```bash
ai-cockpit verify --repo "$repo" --work-item WI-csharp-change \
  --command dotnet --args test
```

其他检查可以作为独立 verification entry 声明。repository 的 `light`、`standard` 或 `strict` 比例 profile
决定哪些检查必需；更强的 Verification Tier 不等于 Evidence Assurance。Hosted CI、provider attestation 和企业
控制仍是委托 evidence。

源建议的 coverage（生产 `src/**`，测试 `tests/**` 或 `**/*Tests/**`）是 policy 思路，不是强制目录布局。
应使用 repository-relative Contract 边界和当前 scope-overlap validator；不确定或不安全的 pattern 必须
fail-closed，不能静默当作不相交。

## 当前 Contract 映射

先生成 not-ready skeleton，分离事实和决定：

```bash
ai-cockpit work-item new --repo "$repo" --id csharp-change --mode code
```

人类 owner 随后必须定义 intent、goal、scope、out-of-scope、acceptance、authority 和所需 evidence，之后
`preflight` 才能进入可执行状态。当前 Rust 字段的映射如下：

- `protocolVersion` 和可选 `contractVersion` 标识 Rust protocol；源 `contractVersion: 2` 不是直接 wire contract。
- `workItemId`、`mode`、`scope`、`guidelines` 保留治理含义，并经过 repository-relative 安全校验。
- `verification` 是描述性的 typed checks，不能替代新鲜执行或授予权限。strict/release 的 `checkpointPolicy`
  提供当前 Agent Risk 下限；不复制源 `ai*` 名称作为第二 registry。
- `acceptanceCriteria`、编号 acceptance evidence 和 `intentAlignment` 保留可观察完成标准，不凭空生成 guideline 结果。
- `authority` 只是 repository-local 声明；人类身份、provider approval 和企业 assurance 仍是外部 evidence。

## 继承边界与非声明

附加的 C# repository 通过自己的 `.ai/` 和 Agent adapter，从共享 Runtime 继承同样的面向读者的 workflow、停止状态、
Outcome 规则和 evidence 边界。repository identity、Contract、snapshot、Work Item 和 evidence 仍由显式 `--repo` 隔离。

本文有意不复制 `install.sh`、`Makefile.ai.stack`、源 Python checks、源 guard YAML 或源 JSON 示例，也不声称本 Rust
repository 已执行 C# adopter 验收。未来多技术栈验收必须单独建立并提供明确 evidence。

[参考源对齐](reference-parity.zh-CN.md)将其记录为语义/文档 parity，而不是源命令或 JSON-wire 兼容。

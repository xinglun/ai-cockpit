---
author: AI Cockpit maintainers
title: "AI Cockpit"
description: "面向 AI 编码代理的 repository 治理：明确范围、验证证据与可审计的人类决定。基于 Rust 构建，提供 CLI 和 Model Context Protocol（MCP）接口。"
audience:
  - adopter
  - contributor
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - repository_governance_layer
---

# AI Cockpit

[English](README.md) | [日本語](README.ja.md)

面向 AI 编码代理的 repository 治理。AI Cockpit 明确工作范围，让验证证据可供审查，并单独记录人类决定。它基于 Rust 构建，提供命令行界面，并通过本地适配器支持 Model Context Protocol（MCP）。

## 当代理说“完成了”

修改留在声明范围内吗？必需检查确实针对当前 repository 执行了吗？审批仍对应正在审查的修改吗？

工作开始前，人类负责人会记录 [Work Item Contract](docs/getting-started/first-work-item.zh-CN.md)，写明任务、代理可以修改的内容、如何判断完成，以及谁有权批准结果。验证会记录这些检查及 repository 状态的证据。人类决定保持明确且可审计；检查通过不代表批准。

每个 repository 都将自己的治理状态保存在 `.ai/` 目录中。

普通的 repository-only 改动从 `start --prepare` 开始。它会记录 Contract，并执行 preflight（检查 repository 和任务是否具备开始条件）。无需人工决定时，它还会保存 checkpoint，即编辑前的 repository 快照。

实现后，`verify` 记录 Contract 指定检查的证据。`finish` 检查这些证据是否对应当前 repository 状态，然后记录 Work Item Outcome。`archive` 保存 Work Item 记录。

保留 verification 和审计证据。遵循当前 Work Item 的 Runtime next action。如果工作只使用本地 branch 或 worktree，`close` 会先记录决定，再执行本地清理。如果 PR、branch 或 worktree 由 provider 管理，应先完成并验证已声明的清理，再执行 `close`。不要删除唯一保存证据的 worktree。具体步骤见[Agent 工作流参考](docs/reference/agent-workflow.zh-CN.md)。

## 运行真实的首次使用流程

Apple Silicon macOS 可用以下命令安装已发布的稳定版 v1.0.0，并校验 archive 的 SHA-256。需要 curl、shasum、tar 和 install：

~~~bash
set -eu
asset=ai-cockpit-v1.0.0-aarch64-apple-darwin.tar.gz
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
curl -fL "https://github.com/xinglun/ai-cockpit/releases/download/v1.0.0/$asset" -o "$asset"
printf '%s  %s\n' 3af024699ffdc14e095273945d55507a950c6c115ae1ea8f228ef5425fb7b4f3 "$asset" | shasum -a 256 -c -
tar -xzf "$asset" ai-cockpit
mkdir -p "$HOME/.local/bin"
install -m 0755 ai-cockpit "$HOME/.local/bin/ai-cockpit"
export PATH="$HOME/.local/bin:$PATH"
ai-cockpit --version
~~~

如需在新终端使用该命令，请将 $HOME/.local/bin 加入 shell 启动配置的 PATH。

此命令仅适用于 Apple Silicon macOS。Linux ARM64 GNU、Linux x86_64 GNU 和 Windows x86_64 的准确制品文件名与校验和见[分发指南](docs/release/distribution.zh-CN.md)。安装后，选择一个你有权操作的 repository。在 POSIX shell 中运行：

~~~bash
repo=/path/to/repository
ai-cockpit --version
ai-cockpit inspect --repo "$repo"
ai-cockpit attach --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
~~~

Windows PowerShell 可设置 repository 路径，并运行相同的首次使用步骤：

~~~powershell
$repo = "C:\path\to\repository"
ai-cockpit --version
ai-cockpit inspect --repo $repo
ai-cockpit attach --repo $repo
ai-cockpit status --repo $repo
ai-cockpit doctor --repo $repo
~~~

**inspect** 读取 repository 事实。**attach** 初始化 repository 本地 `.ai/` 目录以保存治理状态，不安装 Agent 指令或修改全局 MCP 设置。**status** 汇总 repository 状态和 Runtime 兼容性。**doctor** 检查 repository 是否已 attach、协议版本和 Runtime 兼容性。请检查实际输出：仅安装或 attach 并不代表工作已获批准或验证。[首个 Work Item 路线](docs/getting-started/first-work-item.zh-CN.md)说明完整流程。

## 稳定版与可选预发布版

默认使用稳定版 v1.0.0。[Release 页面](https://github.com/xinglun/ai-cockpit/releases/tag/v1.0.0)列出 v1.0.0 的 Apple Silicon macOS、Linux ARM64 GNU、Linux x86_64 GNU 和 Windows x86_64 制品。v1.0.0 没有 Intel macOS、Linux musl 或 Windows ARM64 archive。

macOS ARM64 v1.0.1-rc.1 是供独立试用的可选预发布版，不是默认安装路径；该预发布版本的正式发布验收检查尚未完成。[预发布页面](https://github.com/xinglun/ai-cockpit/releases/tag/v1.0.1-rc.1)。

## 边界

AI Cockpit 记录范围、声明的验证、证据新鲜度和人类决定。它不提供生产 sandbox、不配置 branch protection、不证明外部 provider 身份，也不替代人工 review 或安全策略。检查记录是关于该检查和 repository 状态的证据，并不代表普遍安全或性能提升。

## 继续阅读

- [开始使用](docs/getting-started/README.zh-CN.md)
- [功能与边界](docs/capabilities.zh-CN.md)
- [发布与分发](docs/release/distribution.zh-CN.md)
- [Agent 工作流参考](docs/reference/agent-workflow.zh-CN.md)
- [贡献指南](CONTRIBUTING.md)

---
author: AI Cockpit maintainers
title: "功能与边界"
description: "面向读者说明 AI Cockpit 当前能做什么，以及哪些责任仍在外部。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - cli_lifecycle
  - mcp_adapter
  - bounded_verification
---

# 功能与边界

## 目的

把本页当作当前功能索引。每一行说明用户能做什么、从哪个命令开始，以及会得到
什么状态或证据。

## 开始前

安装或构建 `ai-cockpit`，并把它指向 Git repository。`inspect` 是只读操作；`attach`
是推荐的显式准备步骤，可能创建 `.ai/`；如果缺少协议文件，`start` 也能自动完成
bootstrap。依赖 evidence reuse 前请先检查 attached profile。

## 术语说明

- **snapshot**：一次 repository 观察结果，包括 Git 和相关文件 digest；
- **profile**：经明确确认、可用于受控 reuse 的质量命令列表；
- **receipt**：一次验证结果的、与内容绑定的 evidence；
- **有界验证**：带 worker 上限、超时和有界输出捕获的执行；
- **reuse**：只有所有授权 identity binding 一致时才跳过命令；
- **fail closed**：证据缺失或矛盾时重新执行、返回 unknown 或停止。

## 功能一览

| 功能 | 用户可以做什么 | 从这里开始 | 结果 |
| --- | --- | --- | --- |
| Inspect | 不修改 repository 地读取状态。 | `ai-cockpit inspect --repo <path>` | Git identity、changed paths、digest 和 runtime identity。 |
| Attach | 创建 repository 持有的治理界面。 | `ai-cockpit attach --repo <path>` | `.ai/cockpit.toml`、`.ai/project.json` 和校准状态。 |
| Observe | 读取 profile 并推导 repository 事实。 | `ai-cockpit observe --repo <path>` | observation 与 evolution signal。 |
| Preflight | 编辑前评估 Work Item contract。 | `ai-cockpit preflight --repo <path> --contract <file>` | green、yellow 或 red 的 governance decision。 |
| Work Item 生命周期 | 启动、checkpoint、完成、归档并关闭有界工作。 | `start`、`checkpoint`、`finish`、`archive`、`close` | 明确的状态转换和 receipt。 |
| Verification | 在允许命令和资源限制内运行检查。 | `ai-cockpit verify --repo <path> ...` | pass/fail/unknown 与执行 evidence。 |
| Evidence reuse | 所有 identity binding 一致时才跳过重复运行。 | 已确认 profile + 自动 `verify` | reuse 或 fail-closed rerun。 |
| Knowledge | 查询 repository-local 的已完成 evidence。 | `ai-cockpit knowledge query --repo <path>` | 过滤结果，不是第二事实源。 |
| MCP | 向 MCP client 提供相同 repository service。 | `ai-cockpit mcp --repo <path>` | 有显式绑定的 JSON-RPC 结果。 |
| Doctor | 诊断 runtime 和 repository 准备度。 | `ai-cockpit doctor --repo <path>` | 可操作诊断，不静默修复。 |
| Profile confirmation | 确认可用于受控 reuse 的质量命令。 | `ai-cockpit profile confirm --repo <path> --program cargo --args test,--workspace` | 可审查的新 profile 版本。 |

## 面向用户的详细路径

### 检查 repository

**可以这样理解：**“不修改任何东西，显示 repository 状态。”

```bash
ai-cockpit inspect --repo /path/to/repository
```

命令报告 repository root、Git head、changed paths、tree/diff digest、dependency
fingerprint、读写计数和 runtime identity。如果 discovery 或 Git 失败，应先停止并修复路径。

### Attach 和 Observe repository

**可以这样理解：**“为受治理的 Work Item 准备这个 repository。”

```bash
ai-cockpit attach --repo /path/to/repository
ai-cockpit observe --repo /path/to/repository
```

Attach 可能创建或更新 `.ai/cockpit.toml` 和 `.ai/project.json`，但不会把 Rust source、
V1 runtime file、Python helper 或 runtime schema 复制到目标。初始 profile 是
`calibration_required`；确认质量命令后才可用于受控 reuse：

```bash
ai-cockpit profile confirm --repo /path/to/repository \
  --program cargo --args test,--workspace
```

### Preflight Work Item

`start` 会创建供 `preflight` 读取的 contract：

```bash
ai-cockpit start --repo /path/to/repository --id WI-123 \
  --intent "Improve documentation" \
  --goal "Explain installation clearly" \
  --scope 'docs/**' --authority authorized \
  --acceptance "examples work"
ai-cockpit preflight --repo /path/to/repository \
  --contract .ai/work-items/active/WI-123.contract.json
```

Preflight 针对当前 snapshot 评估 contract。authority 缺失、contract 过期、超出 scope
或事实矛盾都会停止流程。

### 运行受治理的 Work Item

**可以这样理解：**“开始有界修改，记录进度，并只在 review 后关闭。”

```bash
# preflight 可接受后，只编辑 docs/**
ai-cockpit checkpoint --repo /path/to/repository --id WI-123
ai-cockpit verify --repo /path/to/repository --work-item WI-123 \
  --command cargo --args test,--workspace --workers 2
ai-cockpit finish --repo /path/to/repository --id WI-123
ai-cockpit archive --repo /path/to/repository --id WI-123
ai-cockpit close --repo /path/to/repository --id WI-123 \
  --human-decision approved
```

预期状态依次为 `implementation_active`、`checkpointed`、`finish_ready`、`archived`、
`closed`。`finish` 要求同一 Work Item、同一 repository snapshot 的 passed verification
receipt；`close` 要求 archive manifest 和 human decision。检查失败时保留 Work Item，修复
缺失 evidence，不要删除记录。

### Verification 和 reuse

显式命令和绑定 Work Item 的验证总是 fresh：

```bash
ai-cockpit verify --repo /path/to/repository \
  --command cargo --args test,--workspace --workers 2
```

自动检测可使用已确认 profile，并可能复用持久化 receipt：

```bash
ai-cockpit verify --repo /path/to/repository
ai-cockpit verify --repo /path/to/repository
```

第二次结果可能显示 `nodesReused: 1`、`processesSpawned: 0`。只有 repository snapshot、
source/base revision、profile、toolchain、environment、executable identity、scope、
policy、stage、runner、command 和 output identity 全部一致时才能 reuse。protected gate、
显式命令和 Work Item 总是 fresh；不一致会 rerun 或返回明确的 unknown/blocked。

执行限制包括单命令 300 秒超时、stdout/stderr 各 64 KiB、worker 必须为正数。输出可能标记
为 truncated；超时、capture 或 process-tree 失败不能算 pass。receipt-store index 最大 8 MiB，
reusable receipt 最大 1 MiB；malformed、超大、symlink 或不一致的条目 fail closed。

### 查询 knowledge 和 status

```bash
ai-cockpit status --repo /path/to/repository
ai-cockpit knowledge query --repo /path/to/repository --topic installation
```

Knowledge 是 repository-local evidence 的 projection，不是第二事实源。缺失、过期或无效的
Work Item 和 receipt 不能变成新的 claim。

### 使用 MCP

使用显式 repository 绑定启动服务：

```bash
ai-cockpit mcp --repo /path/to/repository
```

服务提供 `status`、`work_item_get`、`work_item_list`、`blockers`、`safe_actions`、
`knowledge_query`、`evidence_get`、`repository_observe`、`preflight`、`verify` 十个工具。
用 `tools/list` 查看 JSON-RPC schema；`preflight` 要求 repository-relative `contract`，
`verify` 接受 `command`、字符串数组 `args` 和可选 `workItemId`。未绑定 repository 的调用
会 fail closed。结果包含 `structuredContent`、文本 content 和 `isError`。CLI 与 MCP 共用同一
套 repository-bound verification policy。

### 诊断准备度

```bash
ai-cockpit doctor --repo /path/to/repository
```

Doctor 报告 runtime version/digest、protocol state、repository identity 和可操作问题。它不是
通用 security scanner，也不声称外部 identity、provider、branch 或生产 control 已满足。

## AI Cockpit 不声称什么

AI Cockpit 不是 Agent Runtime、Workflow Engine、Security Sandbox、通用 prompt-injection detector、
identity provider、合规证书，也不是人工 review 的替代品。外部 identity、branch protection、
生产隔离、签名、SBOM、provenance 和 enterprise policy 仍属于外部证据或采用者责任。

## 停止与恢复

面对缺失或矛盾 evidence，应停止，保留 Work Item 和 receipt，解释缺口，修复相关事实后再 rerun。
绿色的 command output 不能覆盖红色的 governance state。

## 下一步

1. [安装与分发](release/distribution.zh-CN.md)
2. [架构](architecture.zh-CN.md)
3. [设计思想](philosophy.zh-CN.md)
4. [Repository Protocol v1](protocol/v1/specification.zh-CN.md)

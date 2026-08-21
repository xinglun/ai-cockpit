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
  - agent_discovery_adapter
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
| Attach | 创建最小的 repository 治理脚手架。 | `ai-cockpit attach --repo <path>` | `.ai/` 协议文件、discovery manifest、状态目录和校准状态。 |
| Compatibility 与 migration | 检查安装的 Runtime 是否能安全使用 repository，并在需要时应用显式 schema migration。 | `compatibility`、`migrate plan`、`migrate apply --approved` | `COMPATIBLE`、`MIGRATION_REQUIRED` 或 `INCOMPATIBLE`；批准的迁移生成绑定 Runtime 的 receipt。 |
| Observe | 读取 profile 并推导 repository 事实。 | `ai-cockpit observe --repo <path>` | observation 与 evolution signal。 |
| Preflight | 编辑前评估 Work Item contract。 | `ai-cockpit preflight --repo <path> --contract <file>` | green、yellow 或 red 的 governance decision。 |
| Work Item 生命周期 | 启动、checkpoint、完成、归档并关闭有界工作。 | `start`、`checkpoint`、`finish`、`archive`、`close` | 明确的状态转换和 receipt。 |
| Verification | 在允许命令和资源限制内运行检查。 | `ai-cockpit verify --repo <path> ...` | pass/fail/unknown 与执行 evidence。 |
| Evidence reuse | 所有 identity binding 一致时才跳过重复运行。 | 已确认 profile + 自动 `verify` | reuse 或 fail-closed rerun。 |
| Knowledge | 查询 repository-local 的已完成 evidence。 | `ai-cockpit knowledge query --repo <path>` | 过滤结果，不是第二事实源。 |
| MCP | 向 MCP client 提供相同 repository service。 | `ai-cockpit mcp --repo <path>` | 有显式绑定的 JSON-RPC 结果。 |
| Doctor | 诊断 runtime 和 repository 准备度。 | `ai-cockpit doctor --repo <path>` | 可操作诊断，不静默修复。 |
| Profile confirmation | 确认可用于受控 reuse 的质量命令。 | `ai-cockpit profile confirm --repo <path> --program cargo --args test,--workspace` | 可审查的新 profile 版本。 |
| Work Item scaffold | 生成可被验证器读取、但不替人做治理决定的骨架。 | `ai-cockpit work-item new --repo <path> --id <id> --mode <mode>` | `not_ready` Contract、snapshot 事实和待补的人类输入。 |
| Profile proposal | 生成候选 profile amendment，不修改正式 baseline。 | `ai-cockpit profile propose --repo <path>` | 只读的 `candidate`/`proposed` 输出。 |
| Agent adapter | 让选定的 Agent 宿主通过 repository-owned 片段发现本 repository。 | `ai-cockpit agent list/install/doctor --repo <path>` | repository-bound discovery、ownership、状态和安全动作；不修改全局配置。 |

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

Attach 创建最小的 repository-owned scaffold：

```text
.ai/
├── cockpit.toml
├── project.json
├── agent-interface.json
├── work-items/active/
├── work-items/archive/
├── evidence/
├── decisions/
└── knowledge/
```

它不会把 Rust source、V1 runtime file、Python helper、provider instruction 或 runtime schema
复制到目标。初始 profile 是 `calibration_required`；确认质量命令后才可用于受控 reuse：

```bash
ai-cockpit profile confirm --repo /path/to/repository \
  --program cargo --args test,--workspace
```

`agent-interface.json` 是 repository-local discovery fact，记录稳定的 repository identity 和 Runtime
能力；它不是 Agent prompt、provider 安装、授权或全局 MCP 设置。

### Runtime 升级与 repository migration

Runtime 升级和 repository migration 是两件事。兼容的 Runtime 升级不会重写 `.ai/`，也不会产生全局
current repository。先针对显式 repository 检查安装的 Runtime：

```bash
ai-cockpit compatibility --repo /path/to/repository
ai-cockpit migrate plan --repo /path/to/repository
```

如果结果是 `MIGRATION_REQUIRED`，先审查 plan，再显式批准：

```bash
ai-cockpit migrate apply --repo /path/to/repository --approved
```

Migration receipt 记录 source/target schema、迁移前后 digest、Runtime version 和 Runtime digest。
迁移只修改 versioned protocol files 和 migration record；archive Work Item、evidence、decision、
knowledge 保持 byte-for-byte historical record。`INCOMPATIBLE` 会在写入前停止，需要理解该 schema
的 Runtime。

当 attached protocol files 完整存在时，有状态的治理操作（`preflight`、Work Item 创建/生命周期、
`verify`、knowledge/profile 写入、Agent adapter 写入以及受治理的 MCP 调用）必须先得到
`COMPATIBLE`。`MIGRATION_REQUIRED` 或 `INCOMPATIBLE` 会在创建新 record 或 evidence 前停止。
compatibility、migration plan、observe、status 和 doctor 等只读诊断仍可用，以便审查下一步安全操作。

### 显式连接 Agent

`attach` 只创建 repository facts，不修改 `AGENTS.md`、`CLAUDE.md`、`GEMINI.md`、
`.cursor/` 或 home 目录配置。需要 Agent 宿主发现本 repository 时，显式选择 provider：

```bash
ai-cockpit agent list --repo /path/to/repository
ai-cockpit agent install --repo /path/to/repository --provider codex
ai-cockpit agent doctor --repo /path/to/repository --json
```

Adapter 只在选定的 repository surface 写入带 marker 的片段，并写入
`.ai/adapters/<provider>.json`；无关字节会保留。`doctor` 根据当前事实推导
`UNATTACHED`、`DISCOVERY_AVAILABLE`、`VERIFIED`、`DEGRADED` 或 `CONFLICT`，不会把 prompt 当作治理权威。
如果 managed section 被修改或有歧义，`repair` 和 `detach` 会拒绝覆盖：

```bash
ai-cockpit agent repair --repo /path/to/repository --provider codex
ai-cockpit agent detach --repo /path/to/repository --provider codex
```

Discovery、adapter 安装、连接、验证和合规是不同状态。MCP 是可选能力；CLI 在没有 MCP 时仍可用，
这些命令不会修改 provider 的全局配置。

### 创建 Work Item 骨架

在人类决定尚未准备好时使用 scaffold：

```bash
ai-cockpit work-item new --repo /path/to/repository \
  --id payment-refund-guard --mode code
```

命令只自动填充 `repositoryId`、`baseRevision`、`projectProfileDigest` 和 `repositorySnapshotDigest`。
`intent`、`scope`、`acceptanceCriteria`、`authority` 保持空值或 `unknown`；Contract 与 summary 状态为
`not_ready`，不会生成 `passed`、`approved`、`verified` 或 `completed`。CLI 会直接列出已知事实和仍需人类输入的字段。
旧的 `start` 命令仍可用，但会复用同一底层 scaffold writer 并接受显式人类字段。

### 提议 profile amendment

```bash
ai-cockpit profile propose --repo /path/to/repository
```

命令输出只读的 `candidate`/`proposed` amendment，不会改变正式 `.ai/project.json` 的 bytes 或 digest；只有未来
显式的 apply decision 才能修改 baseline。

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

`finish`、`archive` 和 `close` 的 JSON 结果都会包含绑定的 `outcome` 对象。Agent 必须将该
Outcome 作为独立的对话消息显式呈现；仅写入文件或被折叠的结果不能视为交付确认。

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

### 可追溯性、Outcome 与并行准备度

v2 intelligence projection 将事实与推导分开，绝不代替人类填写意图或授权：

```bash
ai-cockpit work-item approach --repo /path/to/repository --id WI-123
ai-cockpit work-item outcome --repo /path/to/repository --id WI-123
ai-cockpit work-item inspect --repo /path/to/repository --id WI-123
ai-cockpit work-item declare --repo /path/to/repository --id WI-123 \
  --depends-on WI-100 --conflicts-with WI-124 --parallelizable
ai-cockpit knowledge query --repo /path/to/repository --v2
ai-cockpit capability show --repo /path/to/repository
ai-cockpit diagnose --repo /path/to/repository --work-item WI-123
```

`approach` 输出观察到的事实、命名后的推导、证据引用和仍未知的人类输入。`outcome` 将已验证的实现证据
与 Human Benefit Report 分开；没有明确声明的用户收益保持为 `unknown`。Capability Registry 区分检测到的能力
与 profile 确认的验证能力，并记录 confidence 和 evidence。`inspect` 在依赖、冲突或 scope 兼容性未被明确知道时
对并行执行 fail closed。Diagnosis 只报告实际测得的 snapshot/verification 成本，不伪装成 benchmark。

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

企业采用者还应阅读[企业治理边界](security/enterprise-governance.zh-CN.md)，了解 assurance level、策略优先级、
委托证据、敏感数据持久化、保留和外部审计导出的边界。

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

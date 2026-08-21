---
author: AI Cockpit maintainers
title: "Repository Protocol v1"
description: "Repository 持有的存储、identity、receipt 和 decision contract。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - protocol_v1
---

# Repository Protocol v1

Repository Protocol v1 是应用 repository 与外部 AI Cockpit runtime 之间稳定的、由
repository 持有的存储边界。它保存事实、决定、证据和生成的 knowledge，但不安装 runtime。

## 目录布局

```text
.ai/
├── cockpit.toml
├── project.json
├── work-items/
│   ├── active/
│   └── archive/
├── decisions/
├── evidence/
│   ├── <work-item>.verification.json
│   └── reuse/
│       ├── index.lock
│       ├── index.json
│       └── receipts/<64 位小写十六进制>.json
└── knowledge/
```

`cockpit.toml` 保存 protocol version 和 repository identity。`project.json` 是 attached
Living Project Profile。Work Item 保存有边界的 intent、contract、summary 和 outcome。
Verification evidence 写入 `.ai/evidence`；跨进程 reusable receipt 以内容寻址方式保存于
`reuse` store。Knowledge 是确定性 projection，不是第二事实源。

reuse index 使用 schema version 1，绑定 `repositoryId`、`profileDigest` 以及从 `nodeId` 到
receipt ID 的映射。receipt 文件名使用 canonical `sha256:<64 位十六进制>` ID 的小写十六进制
部分，以保证跨平台路径兼容。写入者持有 `index.lock`，并通过 `index.pending` 提交 index；
读取者会拒绝不确定、malformed、超大、symlink 或 binding 不一致的 store。runtime 管理的
store 文件不应由采用者手工编辑。

## 带 identity 的记录

当 Contract、verification evidence、archive manifest 或 reusable receipt 需要把决定绑定到
repository 状态时，使用以下字段。Contract 记录：

| 字段 | 含义 |
| --- | --- |
| `protocolVersion` | runtime 理解的 protocol major。 |
| `repositoryId` | 目标 repository 的稳定 identity。 |
| `repositorySnapshotDigest` | 该决定使用的 repository 观察状态。 |
| `baseRevision` / `headCommit` | 有效时，决定使用的 source range。 |
| `projectProfileDigest` | 用于授权的 attached/calibrated profile。 |
| `createdAt` | UTC RFC 3339 创建时间。 |

Runtime 产生的 verification evidence 还记录 runtime version/digest、command result、output
identity、reuse metrics 和最终 snapshot。Knowledge projection、human decision receipt 等记录有
各自 schema，不会默认包含表中的所有字段。

所有 digest 使用 `sha256:<64 位小写十六进制>`。digest 输入使用 canonical JSON：map key 排序、
array 保留语义顺序、timestamp 使用 UTC RFC 3339。

## Reusable receipt schema

Reusable receipt 使用 schema version 2，并拒绝未知字段。稳定字段包括 `receiptId`、`nodeId`、
`passed`、`outputDigest`、创建/过期 epoch seconds，以及 `EvidenceContext`。Context 绑定
content、base/head 和 changed-path digest、environment、command、scope、governance、toolchain、
policy、profile、stage 和 runner。Receipt ID 是 canonical receipt body 的 digest；篡改、失败、
过期、未来时间或任何 binding 不一致都会使候选变为 `unknown` 并触发执行。

Store 的 index 读取上限为 8 MiB，reusable receipt 读取上限为 1 MiB。这些限制是 fail-closed
资源边界，不承诺保留任意大小的输出。

## Contract envelope

Contract 授权 intent 和 effect boundary。它记录 scope、out-of-scope、risk、authority、acceptance、
required evidence、base revision、project profile digest 和 repository snapshot digest；不冻结测试
数量、helper 文件、class 名或其他中间实现细节。

## Decision states

- `green`：证据支持当前有边界的下一步动作；
- `yellow`：证据或 capability 需要调查或人工确认；
- `red`：控制失败、权限缺失或状态非法。

`unknown` evidence 永远不能解释为 pass。Human decision 作为 decision 记录，不能替代独立
verification evidence。

## Evolution

- L0 content evolution 自动吸收；
- L1 verification evolution 扩展现有 verification graph；
- L2 capability evolution 产生 Yellow candidate 与 Profile proposal；
- L3 governance evolution 需要 human decision，未经明确确认不能变为 mandatory gate。

## 兼容性

当前 runtime 接受 protocol major version 1，并在执行 repository material 前拒绝 malformed 或不
支持的版本。所需字段由消费该记录的具体操作验证。可选 capability 不会被静默升级，也不会自动
转换为 pass；不支持的请求仍会明确报错、返回 unknown 或停止。Protocol major migration 必须
单独审查，并保留旧 evidence。

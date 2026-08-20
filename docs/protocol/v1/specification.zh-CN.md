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
└── knowledge/
```

`cockpit.toml` 保存 protocol version 和 repository identity。`project.json` 是当前
Living Project Profile。Work Item 保存有边界的 intent 与 outcome。Evidence 保存
content-addressed receipt 或 delegated provider evidence 的引用。Knowledge 是确定性
projection，不是第二事实源。

## 必需 identity

每个 protocol-bound record 包含 `protocolVersion`、`repositoryId`、
`repositorySnapshotDigest` 和 `createdAt`。Runtime 产生的 evidence 还包含
`runtimeVersion` 与 `runtimeDigest`。历史记录保留决策边界使用的 Project Profile digest。

所有 digest 使用 `sha256:<64 位小写十六进制>`。digest 输入使用 canonical JSON；map
key 排序，array 保留语义顺序，timestamp 使用 UTC RFC 3339。

## Contract envelope

Contract 授权 intent 和 effect boundary。它记录 scope、out-of-scope、risk、authority、
acceptance、required evidence、base revision、project profile digest 和 repository
snapshot digest。它不冻结测试数量、helper 文件、class 名或其他中间实现细节。

## Decision states

- `green`：证据支持当前有边界的下一步动作；
- `yellow`：证据或 capability 需要调查或人工确认；
- `red`：控制失败、权限缺失或状态非法。

`unknown` evidence 永远不能解释为 pass。Human decision 必须作为 decision 记录，不能
替代独立 verification evidence。

## Evolution

- L0 content evolution 自动吸收；
- L1 verification evolution 扩展现有 verification graph；
- L2 capability evolution 产生 Yellow candidate 与 Profile proposal；
- L3 governance evolution 需要 human decision，未经确认不能成为 mandatory gate。

## 兼容性

不支持 protocol major version 1 的实现必须 Red 停止。Runtime 升级只要继续支持
protocol 1，就不应修改 repository 文件。Protocol major migration 必须是单独审查的操作。


# WI-119 — Contract preflight 人工确认门

## 目标

使 Rust Runtime 的 Contract 事前边界与参考源 Agent 流程一致：不确定时暂停实施并要求
human review，不能静默当作 ready。

## 范围

- 增加兼容的 Contract `sources` 与 `verification` 声明字段。
- 不完整 Contract 返回带 `reviewState: needs_human_confirmation` 的 yellow，并保存绑定的 preflight receipt。
- checkpoint 只允许 green 或 `verification_pending` yellow；人工确认 yellow 与 red 必须 fail closed。
- 保持 repository/Work Item/Contract/snapshot 绑定，同步 CLI/MCP 与三语文档。

## 不在范围内

发布、全局 Agent/MCP 配置，以及重写历史归档 Work Item bytes。

## 验收

1. `work-item new` 后执行 `preflight` 不得 ready，并列出人工字段。
2. 脚手架缺少 authority、intent、scope 或 acceptance 时不能 checkpoint。
3. 缺少已声明 verification 时保持 `verification_pending`，只允许继续收集证据。
4. Contract 或 snapshot 变化后必须重新 preflight。
5. CLI 与 MCP 暴露相同的 review state、阻断、未知项和下一步。

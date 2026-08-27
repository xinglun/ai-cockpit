---
author: AI Cockpit maintainers
title: "WI-341——归档 PR 的 Runtime shadow"
workItemId: WI-341-runtime-shadow-archived-state
description: "仅在存在 active Contract 时运行 immutable Runtime shadow，同时保留归档 PR 的普通仓库门禁。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-341-runtime-shadow-archived-state
---

# WI-341——归档 PR 的 Runtime shadow

本 Work Item 使 public Runtime shadow 及其制品上传绑定到 active Contract。
因此，`finish` 与 `archive` 之后已没有 active Contract 的归档 PR，仍执行
普通仓库门禁，但不会因缺少 active Contract 被错误拒绝。

变更仅限于 workflow 条件、对应回归断言和同步参考文档；不改变 Runtime
Core、发布制品、adopter 验收或 provider 配置。

验收由 archive Contract 与 verification evidence 记录；在 close 之前，已审查
的 pull request 仍是 provider-side 边界。

[English](WI-341-runtime-shadow-archived-state.md) ·
[日本語](WI-341-runtime-shadow-archived-state.ja.md)

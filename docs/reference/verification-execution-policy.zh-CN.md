---
author: AI Cockpit maintainers
title: 验证执行缓存策略
description: 让 Cargo 验证缓存可复用且占用可控。
audience: [adopter, contributor, maintainer]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-879-verification-target-policy
---

# 验证执行缓存策略

Runtime 启动 Cargo 验证命令前，会先解析执行环境：

- `CARGO_INCREMENTAL=0`，关闭对有界验证收益很低的增量产物；
- `CARGO_TARGET_DIR` 使用 `$HOME/.cache/ai-cockpit-verify-target`（或平台等价的用户目录），
  让不同 Work Item 复用已编译依赖，而不是每个 checkout 建立一套 `target`；
- 非 Cargo 命令保持其声明的环境不变。

这项策略属于 Runtime 执行边界，不是 shell 约定。有效环境会进入验证观察身份，
因此策略或工具链变化不会静默复用不相关的 receipt。规划仍在任何子进程启动前完成。

验证结束后，先确认没有运行中的 `cargo` 或 `rustc`。随后可以删除仓库本地的增量目录：
`rm -rf target/debug/incremental`；这不会删除共享 target 缓存及其可复用依赖。不要把
`cargo clean` 当作日常清理。

该策略只控制缓存位置，不授予验证、授权、合并、发布或人工批准状态。

---
author: AI Cockpit maintainers
title: "WI-466 — 发布 v0.2.54 与公开 adopter 验收"
workItemId: WI-466-release-v0-2-54
description: "从已审查的主线发布 v0.2.54 Runtime，并在隔离 adopter 流程中验证公开二进制。"
audience: [adopter, maintainer, reviewer]
status: in_progress
authority: authorized
lastVerifiedBy: WI-466-release-v0-2-54
---

# WI-466 — 发布 v0.2.54 与公开 adopter 验收

## 意图

发布包含已关闭 Work Item 文档晋级修复的补丁，并证明公开制品能够在不使用源码或 workspace fallback 的条件下初始化并治理隔离的 adopter 仓库。

## 范围

- 在三种语言路线中将 workspace package identity 与当前安装指引推进到 `v0.2.54`。
- 从已同步的 `main` 推送 annotated tag，运行已审查的发布工作流，保留 manifest、校验和、SBOM、provenance 与 tag 证据。
- 使用隔离的 HOME、XDG_CONFIG_HOME、TMPDIR、CARGO_HOME 与 adopter 仓库安装公开制品，执行 public adopter 与 N-1 验收。
- 保留 `first-adopter-smoke=not_ready`、Runtime/repository identity、证据复用、生命周期收据与清理证明。

## 不在范围内

参考源 checkout、对象工程、全局 Agent/MCP 配置、Homebrew tap 修改、源码 fallback、Runtime 架构重构及无关的参考源比对批次。

## 验收标准

1. Workspace package 与发布文档准确推进到 `v0.2.54`，不改写保留的历史发布记录。
2. 已审查的发布工作流绑定 annotated tag、源码提交、manifest、`SHA256SUMS`、SBOM、provenance 与公开制品身份。
3. 本地 strict、版本、工作流、文档与 workspace 测试通过，且不使用源码 fallback。
4. 合并后由发布后 adopter 验收脚本下载并校验 `v0.2.54` 公开二进制，同时保留 Runtime identity 与清理收据。
5. 关闭后 Runtime 仓库保持健康并为 `ready_on_base`。

## 证据与验证

终态记录将发布 tag 与公开制品绑定到已审查的源码提交。Adopter 证据必须保留 `runtime.json`、仓库与 Work Item identity、生命周期收据、证据复用结果、隔离 manifest 与清理状态。验证命令为：

```text
cargo test --locked --workspace
```

公开发布与 N-1 验收脚本属于发布后证据；失败不得改写 Release truth。

## 边界

`v0.2.54` 是同 schema 补丁。Runtime 升级与 repository attach 仍然分离，发布不会 attach 或修改 adopter 仓库。

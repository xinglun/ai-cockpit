---
title: "WI-625——v0.2.84 发布与对象验收"
description: "发布 direct-merge 恢复修复，并用不可变发布产物完成发布后对象验收。"
author: AI Cockpit maintainers
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-625-release-v0-2-84
lastVerifiedBy: WI-625-release-v0-2-84
---

[English](WI-625-release-v0-2-84.md) · [日本語](WI-625-release-v0-2-84.ja.md)

# WI-625——v0.2.84 发布与对象验收

## 目标

将已审查 Runtime 发布为 `v0.2.84`，包含首次 `direct_merge_no_pr` CLI 往返修复，
再用不可变发布产物执行对象与 N-1 验收。

## 边界

本 Work Item 覆盖版本元数据、发布文档、发布策略与发布后证据。不修改对象工程，
不复制参考源脚手架/Python/Make 实现，也不修改全局 Agent/MCP 配置。

## 验收

1. Workspace 包与 `Cargo.lock` 解析为 `0.2.84`。
2. Release CI 发布带注释的 `v0.2.84` 标签、目标归档、校验和、SBOM/provenance、
   Formula 及匹配的 Runtime identity。
3. Public adopter 与 N-1 只使用不可变的 `v0.2.84`/`v0.2.83` 产物，并证明仓库隔离
   与运行根目录清理。
4. 中英日发布、版本与 parity 记录说明新版本及 `v0.2.83` N-1 边界。
5. 终态 Outcome 面向人显示，并记录状态、未知、证据、人工决定和下一步。

## 场景覆盖

- `v0.2.84 source and artifact identity`：将版本、注释 tag、manifest、归档、SBOM、校验和及 Runtime identity 绑定到同一个已审查提交。
- `public adopter and N-1 upgrade`：只使用不可变的 v0.2.84/v0.2.83 产物，证明仓库隔离和临时根目录清理。
- `direct_merge_no_pr recovery availability`：验证发布 Runtime 的 plan-to-apply 往返，不手工修改生成证据。

## 验证

运行 workspace 测试、文档与发布策略/版本检查，以及公开产物对象/N-1 验收。将下载的
Runtime 版本和 SHA-256 写入发布证据；不得用源码或 workspace binary 代替发布证据。

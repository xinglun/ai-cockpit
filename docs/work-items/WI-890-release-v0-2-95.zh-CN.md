---
author: AI Cockpit maintainers
workItemId: WI-890-release-v0-2-95
title: 四方向收敛后的受治理 v0.2.95 发布
description: 仅在 Outcome、HCI、四方向收敛、Issue #851、接口发现、Rust/工具链和当前性能证据完成验证后发布 v0.2.95。
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-890-release-v0-2-95
---

# WI-890——四方向收敛后的受治理 v0.2.95 发布

本 Work Item 是在 Outcome 交付修复、四方向收敛、Issue #851、BAML 启发的
接口发现试点以及 Rust/工具链升级完成后的最后发布路径。只有评审后的候选
版本合并、不可变 tag 和 Release 存在，并且下载的制品通过发布验收 harness，
发布才算完成。

## 验收边界

- 保留 v0.2.93 作为 N-1 基线，v0.2.95 只能从已评审且同步的 main 提交发布。
- 在隔离根目录中验证发布包、校验和、manifest、Release 制品以及下载后的全新
  安装/N-1 升级验收。
- 明确保留已知未知项：当前性能证据显示没有超出噪声的退化，但没有证明开发
  周期改善；未配置宿主时默认对话展示确认仍未知。
- 不修改对象仓库的 main 分支，不增加产品范围，也不把生成报告当作用户对话
  展示的证明。

## 验证计划

在修改 provider 之前完成声明的格式、治理、文档、打包、候选和发布验收检查。
使用 `CARGO_INCREMENTAL=0` 的共享验证 target，记录命令、退出码、日志、制品
摘要、环境身份和清理证据。验收失败必须保留证据，不能通过重复发布同一 tag
来掩盖失败。

## 发布及发布后证据

评审合并且 hosted checks 全绿后，通过仓库发布 workflow 发布不可变的 v0.2.95
tag/Release。使用 `gh release download` 下载公开制品，并针对下载内容运行验收
harness，而不是使用工作区构建。最终化和关闭前记录安装/升级状态、发布链接、
校验和与 manifest 身份，以及临时根目录的精确清理结果。

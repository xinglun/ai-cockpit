---
author: AI Cockpit 维护者
title: "WI-389——参考文档第 22 批"
workItemId: WI-389-reference-documentation-batch-22
description: "逐一比较六个卸载与升级参考文档，在不复制源 authority 的前提下记录有界 Rust 文档对等。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-389-reference-documentation-batch-22
canonical: docs/work-items/WI-389-reference-documentation-batch-22.md
---

# WI-389——参考文档第 22 批

## 意图与边界

在源提交 `e5acb677da6621004d96f0ef353c58fe8d3acfbf` 下逐一比较以下六个路径。通过当前 Rust 原生的已安装生命周期与升级路线保留面向读者的治理含义，同时不把源 installer 命令、provider authority 或历史结论带入目标仓库。

| 固定参考路径 | 分类 | Rust 对应/有界决定 |
| --- | --- | --- |
| `docs/troubleshooting/uninstall.ja.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.ja.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/troubleshooting/uninstall.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/troubleshooting/uninstall.zh-CN.md` | 有意采用不同实现 | `docs/reference/installed-lifecycle.zh-CN.md` 保留只读盘点、负责人确认、提案与独立执行确认、范围受限移除、回执验证、证据保留和 Unknown 时 fail-closed 恢复。 |
| `docs/upgrade.ja.md` | 有意采用不同实现 | `docs/reference/upgrade.ja.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |
| `docs/upgrade.md` | 有意采用不同实现 | `docs/reference/upgrade.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |
| `docs/upgrade.zh-CN.md` | 有意采用不同实现 | `docs/reference/upgrade.zh-CN.md` 保留不可变 Release/Runtime identity、配置回滚安全、冲突与 downgrade 停止、显式 migration 和单独评审的 `--upgrade-with-active` 恢复。 |

## 验收

- 每个固定文件都已阅读，并在 inventory 中有明确分类和对应物。
- inventory、三语 comparison、parity 记录同步，`migrate-gap` 保持为零。
- 已安装生命周期与升级路线说明先提案后写入、明确人工确认、不可变 Release 绑定、回滚、冲突停止和恢复边界。
- 不复制或提升源 Python/Make 命令、provider authority 或历史证据。
- 明确共享 Runtime 与对象/采用方工程继承边界：一份已安装 binary、显式 `--repo`、隔离的工程事实与证据。
- 文档、inventory、治理和已安装 Runtime 验证检查通过。

## 验证与非声明

这是语义/文档对等，不是源命令、JSON wire 或 provider state 兼容。目标可将卸载责任分布在已安装生命周期与升级路线中；在已记录有界对应物和非声明时，没有同名卸载页面不代表遗漏。

[English](WI-389-reference-documentation-batch-22.md) · [日本語](WI-389-reference-documentation-batch-22.ja.md)

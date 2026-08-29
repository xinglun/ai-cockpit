---
author: AI Cockpit maintainers
title: "WI-383——参考 inventory 计数恢复"
workItemId: WI-383-reference-inventory-count-recovery
description: "在 WI-382 不可变 CI 失败后，补齐参考 inventory 计数交付的三语 parity 注册。"
audience:
  - maintainer
  - reviewer
status: recovered
authority: canonical
lastVerifiedBy: WI-383-reference-inventory-count-recovery
---

# WI-383——参考 inventory 计数恢复

## 意图与边界

WI-383 是不可变 WI-382 的显式恢复 successor。Hosted CI 正确发现 WI-382
修正了三语 comparison 页面，却遗漏了必需的 parity ledger 注册。本 Work Item
保留 WI-382 的全部 Contract、evidence、archive、Outcome 和 recovery bytes，
只补齐遗漏的文档投影。

## 范围与验收

三语 `reference-file-comparison` 页面继续使用来自 5,119 条 inventory 的同一
标记（4,262 generated history、292 implemented different-by-design、1
implemented-equivalent、4 not-applicable、45 reference-only、515
deferred、0 migrate gap）。三语 `reference-parity` 页面必须在写入 verification
evidence 前登记 WI-382 的已恢复行和 WI-383 的进行中行。三语 Work Item 页面
必须保持身份和状态元数据一致，并链接到受治理记录。

不修改 Runtime、protocol、inventory 分类、CI workflow、发布产物或全局
Agent/MCP 配置。参考源 checkout 仅用于语义对照，不复制其文件。

## 验证

使用已安装 Runtime，并为每条 repository 命令显式提供路径；运行 inventory、
文档状态和 governance-integrity 检查。WI-382 保持不可变历史 recovery
前驱；只有 WI-383 successor 在 hosted checks、精确合并、关闭和清理完成后
才能晋级。

[English](WI-383-reference-inventory-count-recovery.md) ·
[日本語](WI-383-reference-inventory-count-recovery.ja.md)

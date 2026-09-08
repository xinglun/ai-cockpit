---
author: AI Cockpit maintainers
title: WI-683——P2-C 生命周期并发与恢复重新交付
description: 从最新默认分支以唯一 Work Item 身份重新交付生命周期并发与恢复边界。
workItemId: WI-683-p2c-lifecycle-concurrency-recovery
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-683-p2c-lifecycle-concurrency-recovery
---

# WI-683——P2-C 生命周期并发与恢复重新交付

本 Work Item 在此前重新交付与无关的 WI-681 发生身份冲突后，从当前
`origin/main` 重新交付 P2-C 边界。WI-677 的 archive、evidence 和 recovery
decision 保持为不可变历史记录；本 Work Item 负责新的实现与验证。

## 边界

`finish`、`archive`、`close`、恢复决定记录和 active artifact reconciliation
共用被忽略的 `.ai/locks/` runtime 目录下、按 Work Item 划分的操作系统文件锁。
锁使用稳定 inode，并在句柄或进程退出时由操作系统释放。这在不引入数据库、全局
仓库缓存或新 crate 的前提下，同时串行化线程和独立 CLI 进程。`atomic_write`
将进程 ID 与既有原子序列号组合，避免同一进程复用临时路径。

锁也覆盖 `finish` 的失败投影持久化以及主转换。失败竞争者会观察已提交状态并返回
业务层拒绝，不会用过期的尝试前快照覆盖成功的兄弟操作。既有 JSON schema、生命
周期名称、授权检查、archive 布局和 predecessor 字节保持不变。

## 受控验证

`lifecycle_concurrency.rs` 覆盖：

- 两个线程 finish 同一 Work Item；
- 两个独立测试进程 finish 同一 Work Item；
- 并发 archive 和 close，并确保每项恰好一次提交；
- active projection 缺失和 archive projection 损坏时，在记录新提交前 fail closed。

并发运行后解析最终 summary、outcome、archive manifest 和 close decision 的 JSON。
原始文件系统竞态错误会被拒绝。任何崩溃一致性限制都必须保持为明确的 recovery 或
unknown 状态；本 Work Item 不会从 projection 推断授权。

## 验证与剩余风险

Rust、格式化、Clippy、文档、治理和 hosted 检查都在精确 successor head 上运行。
该锁只保护使用本仓库实现的进程；绕过仓库 API 的外部写入不在安全声明范围内。

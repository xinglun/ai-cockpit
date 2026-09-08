---
author: AI Cockpit maintainers
title: WI-677——WI-657 生命周期并发与恢复 successor
description: 从当前默认分支重新验证 P2-C，并为并发生命周期提交建立边界。
workItemId: WI-677-wi657-lifecycle-concurrency-successor
audience:
  - contributor
  - maintainer
  - reviewer
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-677-wi657-lifecycle-concurrency-successor
---

# WI-677——WI-657 生命周期并发与恢复 successor

本 successor 从当前 `origin/main` 重新交付 P2-C。已归档的 WI-657 分支修复了
同一进程临时文件名冲突，但基线已经过时，并明确留下了回滚覆盖风险；其证据
保持历史不变，本 Work Item 不改写它。

## 边界

`finish`、`archive`、`close`、恢复决定记录和 active artifact reconciliation
共用按 Work Item 划分的操作系统文件锁，锁位于被忽略的 `.ai/locks/` 运行时目录。
稳定锁文件的 inode 不删除，句柄或进程退出时由操作系统释放锁。因此同一进程的
线程和独立 CLI 进程都被串行化，不引入数据库、全局仓库缓存或新 crate。
`atomic_write` 同时使用进程号和已有的原子序列计数器，避免同一进程复用临时路径。

锁也覆盖 `finish` 的失败投影持久化。竞争失败者会观察到已经提交的状态并返回业务
级拒绝，不能用旧快照覆盖成功兄弟调用。现有 JSON schema、生命周期名称、授权检查
和归档布局保持不变。

## 受控验证

`lifecycle_concurrency.rs` 覆盖：

- 两个线程同时 finish 同一个 Work Item；
- 两个独立测试进程同时 finish 同一个 Work Item；
- 并发 archive 和 close，每个操作只能有一个提交；
- active 投影缺失和 archive 投影损坏时，在新提交前 fail closed。

并发后会解析 summary、outcome、archive manifest 和 close decision，确认仍是有效
JSON，并拒绝原始文件系统竞争错误。未来任何崩溃一致性限制都必须保留为显式恢复
或 unknown 状态；本 Work Item 不从投影推测授权。

## 验证与剩余风险

Rust、格式化、Clippy、文档和治理检查均针对 successor 精确提交运行。锁只保护
使用本仓库实现的进程；绕过仓库 API 的外部写入不在安全声明内。

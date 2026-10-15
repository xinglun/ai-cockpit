---
author: AI Cockpit maintainers
title: "WI-1085 — 旧 v1 关闭证据的显式现时来源登记"
description: "Present-day explicit attestation with preserved legacy history and bounded recovery."
audience:
  - maintainer
  - reviewer
workItemId: WI-1085-legacy-v1-closeout-source-registration
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1085-legacy-v1-closeout-source-registration
---

[English](WI-1085-legacy-v1-closeout-source-registration.md) · [日本語](WI-1085-legacy-v1-closeout-source-registration.ja.md)

# WI-1085 — 旧 v1 关闭证据的显式现时来源登记

此工作项仍在进行。聚焦回归和独立审阅证据已保留；正式 Runtime 验证、验收及终态回执尚未完成。lastVerifiedBy 标识当前工作项投影，不表示已验证关闭。

## 预期收益与边界

维护者可使用独立 checkout 和固定完整 commit，对完整旧 v1 closeout 显式登记当前观察。当前登记者只声明自己核对过的字节；旧 producer、时间、Contract、归档、验证及 provider 原文保留，历史证据不提升为当前 verified。

## 实现

不可变 requirement 保存首次准确 registration 模板，registration 绑定完整且有预算的 raw/canonical inventory。同一 WI lifecycle lock 串行保护 no-replace 发布。root/extension resume 使用独立保存的原始字节和摘要。准确恢复 root 后，extension 不完整时仍为 pending，不接受未绑定 transition。合法 Retained→Deleted 通过显式 record 或普通 finalize 进入同一个持锁写入核心。

损坏的 canonical 文件名及 ASCII 大小写别名继续保留义务；跨 WI 依赖查询有循环、深度与工作预算。必需证据损坏会撤销 closed 和 scope 释放。完整登记后不默认要求原来源 checkout 永久在线，明确的外部依赖仍保留失败语义。若全部登记义务及独立可用登记锚一起丢失或回滚，读取者无法推断过去的登记；没有引入远端单调登记簿。

## 证据与待办

[实施 ledger](../../.ai/evidence/WI-1085-legacy-v1-closeout-source-registration/implementation-20261015/ledger.md) 保留实际 RED/GREEN、编译失败、子进程发布测试、文件绑定、资源采样和审阅发现。崩溃矩阵测试发布原语；完整历史 API 的 partial-state 恢复由另一集成 fixture 覆盖，两者均不是正式 Runtime 回执。

继续完成审阅修正与三语 commands/workflow，取得必需 parity 行的 scope 准入，准备材料审阅及 controls，再使用单构建任务、隔离 target 和 900 秒外层上限执行声明的 workspace 验证。本工作项不授权 PR、Ready、merge、真人 close、发布或清理。

全锚丢失边界按登记代次成立：后续 E/T 一起丢失时，即使较早 R/G 仍在，也可能无法识别该 extension。独立保留的 E 只有显式提交时可使用，status 不自动发现外部副本。

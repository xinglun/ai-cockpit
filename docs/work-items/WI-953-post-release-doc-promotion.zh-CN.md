---
author: AI Cockpit maintainers
title: "WI-953——发布后文档投影晋级"
description: "在 v0.2.102 发布收尾后晋级已关闭 WI-952 的文档投影。"
audience: [maintainer, reviewer, contributor]
status: recovered
authority: human:xinglun
workItemId: WI-953-post-release-doc-promotion
lastVerifiedBy: WI-953-post-release-doc-promotion
---

[English](WI-953-post-release-doc-promotion.md) · [日本語](WI-953-post-release-doc-promotion.ja.md)

# WI-953——发布后文档投影晋级

## 意图

在 v0.2.102 发布路径完成后，晋级已关闭 WI-952 的人类文档和 parity 投影。

## 边界

本 Work Item 只修改生成的收尾记录和人类文档投影。Runtime 行为、发布产物和
对象仓库不在范围内。

## 验收

- WI-952 的三语页面和 parity 行如实表达其已归档、已最终化和已关闭状态。
- 文档检查不启动项目验证子进程。

## 替换记录

该路径在验证前被替换：其不可变 Contract 无条件声明 Cargo workspace 验证命令，
与文档专用边界冲突。归档字节保持不变；有效的 retirement receipt 为
`.ai/decisions/WI-953-post-release-doc-promotion.retirement.json`，WI-954 是
绑定的后继项。WI-953 不宣称已经验证或关闭。

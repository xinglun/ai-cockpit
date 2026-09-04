---
author: AI Cockpit maintainers
title: "WI-561——v0.2.72 发布与公开产物验收"
description: "发布并验收下一版不可变 AI Cockpit Runtime。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-561-release-v0-2-72
lastVerifiedBy: WI-561-release-v0-2-72
---

[English](WI-561-release-v0-2-72.md) · [日本語](WI-561-release-v0-2-72.ja.md)

# WI-561——v0.2.72 发布与公开产物验收

## 目标

从已审查的默认分支发布 v0.2.72，形成不可变 Runtime 基线；随后证明从公开
Release 下载的 binary 可以在本仓库中治理，而不使用源码 checkout 或 workspace
fallback。

## 范围与边界

- 在 English、简体中文和日本語中同步 Cargo 元数据、lockfile 以及当前发布/版本说明。
- 绑定默认分支上已关闭的参考源比对和文档晋级记录。
- 生成并验证五目标 archive、manifest、校验和、SBOM、provenance、attestation 与 Runtime identity。
- 只使用不可变下载产物，在隔离目录中执行公开 adopter 和 N-1 验收，并证明禁止写入目录及临时运行根目录清理。

对象工程、全局 Agent/MCP 配置、Runtime 行为、源模板复制、失败标签重写和无关历史记录不属于本 WI。

## 验收

1. Cargo 元数据、lockfile 和当前发布/版本页面标识 v0.2.72；v0.2.71 保留为紧邻的上一个公开基线。
2. Release CI 生成绑定身份的五目标产物及供应链回执集合。
3. 公开 adopter 和 N-1 验收只使用 v0.2.72 下载产物，证明隔离与清理，并用同一 binary 验证本仓库。
4. 发布从同步且 ready 的默认分支开始，不改变 Runtime 行为、对象工程、全局配置或无关历史 evidence。

## 验证边界

Contract 验收标准以创建时的原文为权威；本地化页面只改变呈现。只有不可变公开
产物和 adopter 回执均验证后，Release 才算验收。对象工程验收属于外部只读交接，
本页不代为宣称。

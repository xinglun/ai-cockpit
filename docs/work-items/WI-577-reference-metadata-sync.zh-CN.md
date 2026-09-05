---
author: AI Cockpit maintainers
title: "WI-577：当前参考比对元数据同步"
description: "让当前比对基线和三语元数据投影绑定经过评审的发布版本。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: canonical
workItemId: WI-577-reference-metadata-sync
lastVerifiedBy: WI-577-reference-metadata-sync
---

[English](WI-577-reference-metadata-sync.md) · [日本語](WI-577-reference-metadata-sync.ja.md)

# WI-577：当前参考比对元数据同步

## 目标

让面向读者的参考比对与 parity 入口和固定本地参考源、经过评审的 Rust 基线及已发布
Runtime identity 保持同步。使用一个小型、已纳入版本控制的元数据旁车文件作为这些当前
事实及台账计数的单一事实源。

## 范围与边界

范围包括六个三语参考页面、元数据旁车文件、可执行元数据回归测试及其文档验收接入，
以及本 Work Item 的三语页面。历史批次段落和生成的治理证据保持追加式不变。Runtime 行为、
对象工程、全局 Agent/MCP 配置和参考源实现复制均在范围之外。

## 验收

- 六个参考页面使用相同的当前源提交、元数据旁车文件和 `lastVerifiedBy`。
- Rust 当前基线与已发布 Runtime 版本/digest 与旁车文件一致；台账计数由检查推导而非手工填写。
- 过期表头、计数、源锁或译文漂移时，CI 必须 fail-closed。
- 不增加语义分类，不重写历史证据或对象工程。

## 验证

详见 active Contract 与 `tests/docs/reference_comparison_metadata_test.py`。有界检查包括参考
台账、文档验收、Work Item 状态一致性和 `git diff --check`。

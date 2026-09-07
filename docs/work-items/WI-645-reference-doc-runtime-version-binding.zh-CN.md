---
author: AI Cockpit 维护者
title: WI-645——参考源比对 Runtime 版本绑定
description: 让当前参考源比对文档中的 Runtime 投影与机器可读 metadata 保持同步。
audience: [maintainer, reviewer, adopter]
workItemId: WI-645-reference-doc-runtime-version-binding
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-645-reference-doc-runtime-version-binding
---

# WI-645——参考源比对 Runtime 版本绑定

[English](WI-645-reference-doc-runtime-version-binding.md) · [日本語](WI-645-reference-doc-runtime-version-binding.ja.md)

## 意图

修正六个三语参考源比对页面当前快照中的 Runtime 版本，并增加回归断言，
将面向人的投影绑定到 `reference-comparison-metadata.json`。

## 边界

本 Work Item 只修改当前文档投影及其静态 metadata 检查。不改变 Runtime 行为、
Protocol bytes、不可变历史、固定参考源 checkout 或任何对象工程。

## 验收与验证

- 六个页面当前快照中的 Runtime 版本和二进制摘要与 metadata sidecar 完全一致且恰好出现一次。
- metadata 测试在当前快照版本过期时失败，在 metadata 正确时通过。
- 三语文档、inventory、治理完整性和 workspace 检查通过，且不修改历史记录。

Work Item 关闭后，归档页面会链接权威 Contract、verification、finalization 和 decision 记录。

---
author: AI Cockpit maintainers
title: "WI-964 — 不可变归档边界恢复"
description: "在源头修复归档格式，并把可修改路径的 whitespace 校验与不可变归档完整性分开。"
audience: [maintainer, reviewer, contributor]
status: in_progress
authority: human:sei-rinn
workItemId: WI-964-archive-boundary-recovery
lastVerifiedBy: WI-964-archive-boundary-recovery
---

[English](WI-964-archive-boundary-recovery.md) · [日本語](WI-964-archive-boundary-recovery.ja.md)

# WI-964 — 不可变归档边界恢复

此后继保留 WI-963 的失败验证记录。它在生成端修复 task-report Markdown 的 EOF
格式，并只对可修改候选路径执行 whitespace 校验。不可变归档字节继续由 digest
与 archive-integrity 检查治理；本 WI 不重写它们，也不声明发布或收尾已完成。

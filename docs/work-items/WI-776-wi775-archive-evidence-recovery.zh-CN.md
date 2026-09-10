---
author: AI Cockpit maintainers
title: "WI-776——WI-775 archive-evidence recovery"
description: "完成 WI-775 在 verification 与 archive 之间产生 stale evidence 后的有界恢复。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-for-successor-after-governed-archive-failure
workItemId: WI-776-wi775-archive-evidence-recovery
lastVerifiedBy: WI-776-wi775-archive-evidence-recovery
---

[English](WI-776-wi775-archive-evidence-recovery.md) · [日本語](WI-776-wi775-archive-evidence-recovery.ja.md)

# WI-776——WI-775 archive-evidence recovery

## 意图与边界

WI-776 是不可变 yellow WI-775 archive 的明确 successor。WI-775 的通过验证因 verification
与 archive 之间提交导致仓库快照前进而变为 stale。本 Work Item 保留 WI-775、PR #758 与所有
前置字节，并以新鲜 evidence 完成相同的文档/治理投影。

不修改 Runtime 源码、产品行为、授权语义、退出码、性能实现或历史 evidence。

## 验收

- WI-775 保持不可变，并通过 recovery decision 关联。
- 英文、简体中文、日文 WI-775/WI-776 页面与 parity 行匹配 Runtime 状态，不虚构终态 evidence。
- parity 注册先于 reviewed PR 上的新鲜 verification evidence。
- WI-776 完成完整 Runtime lifecycle 与精确清理。

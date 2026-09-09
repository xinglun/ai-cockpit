---
author: AI Cockpit maintainers
title: "WI-714——WI-712 文档晋级"
description: "根据不可变治理证据晋级已关闭 WI-712 的文档投影。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: authorized
workItemId: WI-714-wi712-doc-promotion
lastVerifiedBy: WI-714-wi712-doc-promotion
---

[English](WI-714-wi712-doc-promotion.md) · [日本語](WI-714-wi712-doc-promotion.ja.md)

# WI-714——WI-712 文档晋级

## 意图

根据不可变的 archive、verification、finalization 和 close 证据，晋级已关闭
WI-712 recovery Work Item 与三语 parity 投影。本有界文档 Work Item 不改变
Runtime、治理行为、生产行为、性能行为或历史证据，也不触碰其他 agent 的 Work Item。

## 证据边界

- Archive：`.ai/work-items/archive/WI-712-wi702-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-712-wi702-finalization-recovery.verification.json`
- Finalization：`.ai/decisions/WI-712-wi702-finalization-recovery.finalize.json`
- Close：`.ai/decisions/WI-712-wi702-finalization-recovery.close.json`

该投影保留 WI-702 的不可变 recovery boundary，不记录性能收益。

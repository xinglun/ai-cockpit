---
author: AI Cockpit maintainers
title: "WI-761——WI-760 parity-order recovery successor"
description: "在新 verification evidence 之前提交 parity registration，重新交付 WI-760 文档边界。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: explicit-user-authorization-for-successor-after-hosted-quality-failure
workItemId: WI-761-wi760-parity-order-recovery
lastVerifiedBy: WI-761-wi760-parity-order-recovery
---

[English](WI-761-wi760-parity-order-recovery.md) · [日本語](WI-761-wi760-parity-order-recovery.ja.md)

# WI-761——WI-760 parity-order recovery successor

WI-761 是 WI-760 PR #743 的有界 successor。Hosted governance 发现 parity row
与 verification evidence 在同一提交中产生；WI-760 的失败交付和证据保持不变，WI-761
从最新默认分支重新交付，并先提交三语 parity registration，再生成新的 verification
evidence。

本 Work Item 只修改文档 projection 及其治理记录，不改变 Runtime、产品代码、授权语义、
退出码或 WI-760 的历史证据。

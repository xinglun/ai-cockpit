---
author: AI Cockpit maintainers
title: "WI-760 — WI-759 documentation repair successor"
description: "WI-759 に不足している self-projection pages を補い、successor documentation boundary を bind します。"
audience: [maintainer, reviewer, adopter]
status: recovered
authority: explicit-user-authorization
workItemId: WI-760-wi759-doc-repair
lastVerifiedBy: WI-760-wi759-doc-repair
---

[English](WI-760-wi759-doc-repair.md) · [简体中文](WI-760-wi759-doc-repair.zh-CN.md)

# WI-760 — WI-759 documentation repair successor

## Intent

WI-759 の reviewed PR #742 merge 後に判明した self-projection pages の不足を修復します。
WI-759 の immutable evidence は保持し、pending documentation boundary を明示します。

## Boundary

この Work Item は指定された三言語 documentation pages と reference-parity projection
だけを変更します。Runtime behavior、product code、machine contracts、authorization
semantics、exit codes、WI-759 の historical evidence は変更しません。

## Acceptance

- WI-759 と WI-760 に正確な三言語 pages が存在します。
- parity rows が local Work Item pages と関連する immutable evidence に bind されます。
- pre-archive の documentation、governance、status checks が通り、Runtime が close
  を生成する前に terminal claim を行いません。

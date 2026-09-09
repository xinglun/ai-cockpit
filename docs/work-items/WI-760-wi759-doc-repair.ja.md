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

## Supersede の完了

WI-760 は immutable な失敗 delivery のままです。PR #743 の hosted quality failure、archive、
verification、元の recovery は変更しません。WI-761 が parity-first successor を完了したため、
WI-760 は append-only recovery と close により supersede されました。

- supersede recovery：`.ai/decisions/WI-760-wi759-doc-repair.recovery.59a48d37bdbc566a0dc76635fa4d5ebdcf9ea324cda33322cf0ddc566c644907.json`
- close：`.ai/decisions/WI-760-wi759-doc-repair.close.json`
- successor：WI-761、[PR #744](https://github.com/xinglun/ai-cockpit/pull/744)

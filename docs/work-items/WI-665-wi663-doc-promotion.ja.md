---
author: AI Cockpit maintainers
title: "WI-665 — WI-664 の documentation promotion"
description: "不変の terminal evidence に基づいて WI-664 の documentation projection を昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-665-wi663-doc-promotion
lastVerifiedBy: WI-665-wi663-doc-promotion
---

[English](WI-665-wi663-doc-promotion.md) · [简体中文](WI-665-wi663-doc-promotion.zh-CN.md)

# WI-665 — WI-664 の documentation promotion

## Intent

WI-664 の三言語 Work Item page と共有 reference-parity row を、不変の
archive、verification、finalization、close record に同期します。

## Boundary

これは documentation-only projection です。変更対象は Contract で指定した九つの
Markdown file に限定します。不変の `.ai` archive、evidence、recovery、finalization、
close record は read-only input とし、Runtime、repository、過去の Work Item、他 agent
の挙動は変更しません。

## Acceptance

- WI-664 の page および parity row が verified close 後の terminal `Implemented`
  status と正確な terminal evidence path を示すこと。
- WI-665 がこの bounded documentation projection の明示的な prearchive self-registration
  として監査可能であること。
- promotion、documentation、parity、status consistency、governance、Hosted quality
  check が exact reviewed head で成功すること。

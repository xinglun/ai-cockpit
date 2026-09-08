---
author: AI Cockpit maintainers
title: "WI-661 — WI-660 terminal documentation promotion"
description: "不変の terminal evidence に基づいて WI-660 の documentation projection を昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-661-wi660-doc-promotion
lastVerifiedBy: WI-661-wi660-doc-promotion
---

[English](WI-661-wi660-doc-promotion.md) · [简体中文](WI-661-wi660-doc-promotion.zh-CN.md)

# WI-661 — WI-660 terminal documentation promotion

## Intent

WI-660 の三言語 Work Item page と reference-parity row を、不変の archive、
verification、finalization、close record に同期します。

## Boundary

これは documentation-only projection です。変更対象は Contract で指定した六つの
Markdown file に限定します。不変の `.ai` archive、evidence、finalization、close
record は read-only input とし、Runtime、repository、過去の Work Item の挙動は変更しません。

## Acceptance

- 三言語の WI-660 page が verified close 後の terminal `Implemented` status と正確な
  terminal evidence path を示すこと。
- 三つの WI-660 parity row が対応する terminal status と evidence path を示すこと。
- promotion、documentation、parity、status consistency、governance、Hosted quality
  check が exact reviewed head で成功すること。

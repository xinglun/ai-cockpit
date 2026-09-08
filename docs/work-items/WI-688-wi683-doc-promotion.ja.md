---
author: AI Cockpit maintainers
title: "WI-688 — WI-683 terminal documentation promotion"
description: "不変の terminal evidence に基づいて、検証済み WI-683 の documentation projection を昇格します。"
audience: [maintainer, reviewer, adopter]
status: in_progress
authority: human:repository-owner
workItemId: WI-688-wi683-doc-promotion
lastVerifiedBy: WI-688-wi683-doc-promotion
---

[English](WI-688-wi683-doc-promotion.md) · [简体中文](WI-688-wi683-doc-promotion.zh-CN.md)

# WI-688 — WI-683 terminal documentation promotion

## Intent

WI-683 の三言語 Work Item page と reference-parity row を、不変の archive、
verification、finalization、close record に同期します。

## Boundary

これは documentation-only projection です。不変の governance record は read-only
input とし、Runtime、repository、過去の Work Item の挙動や bytes は変更しません。

## Acceptance

- 三言語の WI-683 page が verified close 後の terminal status と正確な evidence path を示すこと。
- 三つの WI-683 parity row が対応する terminal status と evidence path を示すこと。
- promotion、documentation、parity、status consistency、governance、Hosted quality check が exact reviewed head で成功すること。

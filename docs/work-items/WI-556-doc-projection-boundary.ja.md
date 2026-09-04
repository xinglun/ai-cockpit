---
author: AI Cockpit maintainers
title: "WI-556 — bounded documentation projection"
description: "クローズ済みリリース作業に対する有限で明確なドキュメント投影境界を記録します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-556-doc-projection-boundary
lastVerifiedBy: WI-556-doc-projection-boundary
---

[English](WI-556-doc-projection-boundary.md) · [简体中文](WI-556-doc-projection-boundary.zh-CN.md)

# WI-556 — bounded documentation projection

## Objective

クローズ済み作業のドキュメント投影を有限かつ正確な範囲に固定し、後続 WI の無限連鎖を防ぎます。

## Boundary

Contract が指定する三言語の WI ページと三つの reference-parity ファイルだけを対象にします。Runtime、source、CI、証跡、対象リポジトリは対象外です。

## Acceptance

- 六つの正確なドキュメントパスが archive 前に登録され、終端証跡と一致する。
- クローズ済み Work Item の promotion check が bounded self-projection として扱う。

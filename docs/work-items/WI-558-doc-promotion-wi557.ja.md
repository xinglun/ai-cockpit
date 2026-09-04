---
author: AI Cockpit maintainers
title: "WI-558 — WI-557 の terminal documentation projection"
description: "不変の終端証拠に基づき WI-557 の文書を昇格し、この bounded projection 自身も登録します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: in_progress
authority: canonical
workItemId: WI-558-doc-promotion-wi557
lastVerifiedBy: WI-558-doc-promotion-wi557
---

[English](WI-558-doc-promotion-wi557.md) · [简体中文](WI-558-doc-promotion-wi557.zh-CN.md)

# WI-558 — WI-557 の terminal documentation projection

## Objective

不変の archive、verification、finalization、close receipt に基づき、三言語の
WI-557 Work Item ページと対応する reference-parity 行を条件付き投影から
決定的な終端形へ昇格します。この Work Item 自身の三言語ページも bounded
self-projection として登録し、close 後の無限な documentation successor を防ぎます。

## Scope

- WI-557 の English、簡体中文、日本語 Work Item ページ。
- この Work Item 自身の三言語ページ。
- English、簡体中文、日本語の reference-parity 行。

## Boundary

終端投影を書き込めるのは公式 promotion helper だけです。Runtime、protocol、
reference checkout、対象リポジトリ、無関係な文書は変更しません。self-projection
はこの正確な bounded documentation scope に限られ、evidence validation を迂回しません。

## Acceptance

- WI-557 のページと parity 行が三言語で終端 evidence binding と `Implemented` 状態になる。
- closed Work Item promotion check、documentation acceptance、宣言した検証コマンドが通る。
- この Work Item のページは close 前は条件付き登録のままとし、close 後の gate はこの正確な self-projection だけを終端として認識する。
- 不変 receipt、Runtime の挙動、無関係な projection を変更しない。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo . --work-item WI-557-reference-file-comparison-batch-41 --check`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/documentation_acceptance.sh`
- `git diff --check`

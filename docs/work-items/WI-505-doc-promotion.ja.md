---
author: AI Cockpit maintainers
title: "WI-505 — WI-504 の terminal documentation projection"
description: "close 後の gate が検出した conditional status を修正し、WI-504 の文書と parity projection を terminal に昇格します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-505-doc-promotion
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-505-doc-promotion
---

# WI-505 — WI-504 の terminal documentation projection

[English](WI-505-doc-promotion.md) · [简体中文](WI-505-doc-promotion.zh-CN.md)

## Boundary

この限定 documentation Work Item は、WI-504 の close 後に必須 promotion gate が
報告した問題を消費します。3 言語の Work Item page と parity projection だけを
terminal status へ更新し、closed Work Item を reader-facing baseline に正しく
表します。Runtime が生成した evidence は書き換えず、Runtime behavior も変更しません。

## Scope

- WI-504 の English、簡体中文、日本語 page を evidence-backed terminal status へ昇格する。
- 3 言語の reference parity row を conditional status から `Implemented` へ昇格する。
- projection 更新後に documentation と status-consistency gate を再実行する。

## Out of scope

Runtime source、test、object/adopter repository、reference-source 実装、release
publication、global Agent/MCP configuration、historical evidence または archive の書換え。

## Acceptance

- WI-504 の 3 言語 page が terminal evidence path と `status: implemented` を持つ。
- WI-504 の 3 parity row が `Implemented` で、正確な terminal record にリンクする。
- `promote_closed_work_item.py --repo <repo> --check-all` が成功する。
- documentation、parity、Work Item status-consistency check が成功する。
- 生成 evidence と historical bytes を編集または削除しない。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

terminal projection の source は helper とし、生成 receipt は immutable に保持します。

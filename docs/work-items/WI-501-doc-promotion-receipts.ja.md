---
author: AI Cockpit maintainers
title: "WI-501 — WI-500 terminal documentation と receipt の昇格"
description: "close 済み WI-500 の recovery evidence と生成 receipt を review 済み documentation baseline に昇格します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-501-doc-promotion-receipts
status: in_progress
authority: human-authorized
lastVerifiedBy: WI-501-doc-promotion-receipts
---

# WI-501 — WI-500 terminal documentation と receipt の昇格

[English](WI-501-doc-promotion-receipts.md) · [简体中文](WI-501-doc-promotion-receipts.zh-CN.md)

## Boundary

この限定 documentation Work Item は、close 済み WI-500 の recovery evidence と
Runtime が生成した recovery、finalization、close receipt を review 済み mainline
documentation baseline へ昇格します。historical bytes は書き換えず、Runtime
behavior も変更しません。

## Scope

- 生成済み WI-496/WI-500 recovery/close receipt を byte-for-byte で追跡します。
- WI-500 の 3 言語ページと parity entry を evidence-backed な terminal status
  へ昇格します。
- 本 Work Item 自身の 3 言語ページと parity entry も同じ bounded lifecycle に
  保持し、post-close documentation check を self-terminal にします。

## Out of scope

Runtime source、test、object/adopter repository、reference-source 実装、release
publication、global Agent/MCP configuration、source fallback binary、history rewrite。

## Acceptance

- Runtime 生成の WI-496/WI-500 receipt 5 件を手編集せず byte-for-byte で追跡する。
- WI-500 の 3 言語ページと parity entry が archive、verification、finalization、
  close evidence にリンクする。
- 本 Work Item の 3 言語ページが promotion boundary と receipt provenance を説明する。
- close 後の Work Item promotion、documentation、parity、status-consistency、diff
  check が成功する。
- review 済み PR を merge し、正確な branch/worktree cleanup を記録する。

## Verification

- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `python3 tests/docs/work_item_status_consistency.py --repo <repo>`
- `bash tests/docs/documentation_acceptance.sh`
- `bash tests/docs/parity_status_check.sh`
- `git diff --check`

receipt は Runtime が生成し、追跡後も immutable に保持します。

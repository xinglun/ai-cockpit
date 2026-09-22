---
author: AI Cockpit maintainers
title: "WI-993 — WI-867 terminal documentation projection"
description: "self-terminal check に残った古い状態を修正し、WI-867 の三言語 terminal documentation projection を完了します。"
audience: [maintainer, reviewer, contributor]
status: implemented
authority: authorized
workItemId: WI-993-wi867-doc-promotion
lastVerifiedBy: WI-993-wi867-doc-promotion
terminalArchive: .ai/work-items/archive/WI-993-wi867-doc-promotion.contract.json
terminalVerification: .ai/evidence/WI-993-wi867-doc-promotion.verification.json
terminalDecision: .ai/decisions/WI-993-wi867-doc-promotion.close.json
---

[English](WI-993-wi867-doc-promotion.md) · [简体中文](WI-993-wi867-doc-promotion.zh-CN.md)

# WI-993 — WI-867 terminal documentation projection

WI-993 は WI-867 の範囲を限定した terminal projection を完了します。WI-867 の
archive、verification、close evidence は保持し、三言語のドキュメント投影と
parity ledger だけを更新します。

## Boundary

対象は WI-867 の三言語ページ、本 Work Item の三言語ページ、三つの
reference-parity ledger、および Runtime が生成する `.ai/` evidence だけです。
ソースの動作、release の動作、過去の governance bytes は対象外です。

## Acceptance

- WI-867 の三言語ページと parity row が不変の terminal evidence と一致すること。
- close まで WI-993 自身の三言語 projection を限定的に保持すること。
- WI-867 の単体 projection、リポジトリ全体の `--check-all`、parity、status-consistency がすべて成功すること。

---
author: AI Cockpit maintainers
title: "WI-342 — reference documentation、distribution、enterprise boundary"
workItemId: WI-342-reference-documentation-batch-13
description: "pinned reference の次の 10 path を一つずつ比較し、source history や wire format をコピーせず evidence-backed な Rust counterpart を記録する。"
audience: [maintainer, reviewer]
status: in_progress
authority: canonical
lastVerifiedBy: WI-342-reference-documentation-batch-13
capabilityClaims:
  - reference_parity
---

# WI-342 — reference documentation、distribution、enterprise boundary

## Intent と boundary

この Work Item は pinned reference commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` の 10 path を一つずつ比較します。
target の semantic responsibility と意図した different-design または reference-only
boundary を記録し、reference の Python、Make、adopter record、provider claim、JSON wire
format はコピーしません。

対象は distribution、documentation architecture/authority、documentation context、
enterprise control、external identity です。変更対象は comparison ledger、tri-language
parity documentation、この Work Item の reader-facing record に限定します。Runtime behavior、
release publication、adopter acceptance、global Agent/MCP configuration、後続の reference path は
out of scope です。

## File-by-file decision

Pinned path と evidence-backed decision は
`tests/conformance/reference_file_inventory.json` と tri-language の
`docs/reference/reference-file-comparison*` ledger に記録します。8 path は
`implemented-different-by-design`、source 固有の control/context record 2 path は
`reference-only` です。equivalent、deferred、missing として暗黙に扱う path はありません。

Target は object/adopter boundary を継承します。shared Runtime、明示的な repository context、
repository ごとの `.ai/` isolation、external provider evidence を使い、enterprise identity や
compliance を local に主張しません。Contract/source text が authority であり、localized
presentation は governance fact を書き換えません。

## Acceptance

- Listed path は pinned inventory にそれぞれ一度だけ現れ、evidence-backed classification と
  valid counterpart、または明示的な reference-only boundary を持ちます。
- English、Simplified Chinese、日本語の comparison/parity page が同じ semantic/non-wire
  decision と current ledger count を示します。
- Source plan/context metadata と source adopter control observation を Runtime にコピーせず、
  target evidence として扱いません。
- Inventory、documentation、repository gate が通り、generated history と immutable evidence は
 変更しません。

[English](WI-342-reference-documentation-batch-13.md) ·
[简体中文](WI-342-reference-documentation-batch-13.zh-CN.md)

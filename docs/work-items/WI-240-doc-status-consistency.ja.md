---
author: AI Cockpit maintainers
title: "WI-240 — Documentation status と reference truth consistency"
workItemId: WI-240-doc-status-consistency
description: "Historical governance bytes を書き換えず、Work Item status、reference inventory、parity、release claim を現在の repository evidence に bind する。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-240-doc-status-consistency
authority: canonical
---

# WI-240 — Documentation status と reference truth consistency

本 Work Item は v0.2.31 Runtime と `origin/main` comparison baseline で
documentation truth を更新します。archived Contract、evidence、decision、公開済み
Release record の再解釈や書き換えは行いません。

## Acceptance boundary

- Reference inventory は target commit
  `1c988ce9b04c3dcd45843f6577ed321457eeca0e` に bind し、checkout-only drift を
  無視して、4 capability/profile `migrate-gap` と 720
  `deferred-next-batch` records を正確に保持します。
- 英語、簡体字中国語、日本語の Work Item documents は identity、projected status、
  verifier が一致します。Terminal projection には repository-bound archived Contract
  と close/recovery evidence が必要で、曖昧な cross-document verifier semantics は
  推測せず unknown のままにします。
- Historical recovery は evidence-bound display projection を許可します。
  `Recovered` は `historical` または `recovered`、`Implemented` は本文が immutable
  recovery history を明示する場合だけ `recovered` にできます。
- Provider は `immutable: false` と報告するため、release docs は v0.2.31 を
  identity-bound かつ drift-detectable と記述します。Repository-persisted adopter
  baseline は `aarch64-apple-darwin` で、hosted Linux workflow artifact は短期の
  external evidence のままです。

## Evidence

Deterministic inventory、documentation acceptance、Work Item status regression は
`.ai/evidence/WI-240-doc-status-consistency.verification.json` と archived Work Item
manifest に bind されます。未解決の 4 file-level gap は machine-readable inventory
で可視のまま保持し、documentation projection で close しません。

## References

- [Reference file comparison](../reference/reference-file-comparison.ja.md)
- [Reference source parity](../reference/reference-parity.ja.md)
- [Release distribution](../release/distribution.ja.md)

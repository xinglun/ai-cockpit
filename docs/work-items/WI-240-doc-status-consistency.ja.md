---
author: AI Cockpit maintainers
title: "WI-240 — documentation status と reference truth の整合"
workItemId: WI-240-doc-status-consistency
description: "immutable な PR delivery が hosted governance で失敗した後、WI-245 が documentation-governance delivery を回復する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-245-doc-status-parity-recovery
authority: canonical
---

# WI-240 — documentation status と reference truth の整合

WI-240 は古い default-branch base で verified archive と canonical pre-merge
finalization を生成しましたが、PR #194 は merge されませんでした。その後の release、
parity、close record が `main` を進めたことで、hosted governance は immutable failed
delivery boundary を検出しました。archived Contract、Summary、Outcome、events、
verification、finalization bytes は retained predecessor branch 上の historical truth の
ままであり、この文書はそれらを import も rewrite もしません。

Runtime-generated successor receipt
`.ai/decisions/WI-240-doc-status-consistency.recovery.json` は predecessor の正確な
digest を bind し、適用可能な status、inventory、release-truth delivery を
`origin/main@87bfd866` 上の WI-245 に委譲します。

## Recovery boundary

- PR #194 は superseded として close され、merge されていません。
- WI-245 は implementation content のみ replay し、WI-240 lifecycle record は replay しません。
- pinned public reference commit は変更しません。
- intervening Work Item の provider、release、SBOM、parity、terminal-decision truth を保持します。

## References

- [WI-245 successor](WI-245-doc-status-parity-recovery.ja.md)
- [Reference file comparison](../reference/reference-file-comparison.ja.md)
- [Reference source parity](../reference/reference-parity.ja.md)
- [Release distribution](../release/distribution.ja.md)

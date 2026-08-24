---
author: AI Cockpit maintainers
title: "WI-253 — close 後の documentation terminalization"
workItemId: WI-253-docs-terminalization
description: "immutable close evidence から WI-252 docs を terminalize し、新たに closed となった Work Item の conditional status を拒否する。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-253-docs-terminalization
terminalArchive: .ai/work-items/archive/WI-253-docs-terminalization.contract.json
terminalVerification: .ai/evidence/WI-253-docs-terminalization.verification.json
terminalFinalization: .ai/decisions/WI-253-docs-terminalization.finalize.1ccec42e056dd7eac857ba49d1dc2becd6e2ba21f6461a62599e18101d986293.json
terminalDecision: .ai/decisions/WI-253-docs-terminalization.close.json
authority: canonical
---

# WI-253 — close 後の documentation terminalization

WI-253 は正しく closed となった WI-252 の bounded Runtime successor です。
recovery decision は WI-252 の canonical Contract、Summary、Outcome、Events digests
と、archive、verification、sequence-2 finalization、structured close evidence を
binding します。これらの immutable record は一切編集しません。

## Acceptance boundary

- WI-252 の English、Simplified Chinese、Japanese Work Item docs と
  reference-parity rows は terminal `implemented` / `Implemented` truth を使用し、
  persisted terminal evidence の正確な path を参照します。
- status-consistency regression は、新たに governance 対象となる terminal Work Item
  の各 language counterpart に残る conditional lifecycle wording を拒否します。
  WI-252 enforcement boundary より前の historical docs は遡及的に rewrite しません。
- reference inventory の target working-tree count warning は意図的な negative fixture
  の結果です。canonical count/digest は pinned commit に normalized されたままなので、
  production checker は変更しません。

## Verification と lifecycle

focused regression はまず各 language の conditional wording が受理されることを示し、
次に実際の stale WI-252 projection で失敗し、terminal evidence の projection 後にだけ
通過します。この active registration は future WI-253 archive、verification、
finalization、close path を列挙しますが、それ自体は terminal evidence ではありません。

## References

- [WI-252 predecessor](WI-252-manifest-gate-order-recovery.ja.md)
- [Reference parity](../reference/reference-parity.ja.md)


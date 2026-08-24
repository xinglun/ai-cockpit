---
author: AI Cockpit maintainers
title: "WI-244 — Pending parity registry"
workItemId: WI-244-pending-parity-registry
description: "別の documentation change で提供する parity row のために、型付きで fail-closed な pre-merge registry を追加します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-244-pending-parity-registry
authority: canonical
---

# WI-244 — Pending parity registry

code Work Item は有効な archive と pre-merge finalization に到達しても、三言語 parity
ledger を編集する authority を持たない場合があります。同じ PR に row を要求すると scope
と finalization head が deadlock します。WI-244 は predecessor の `.ai` bytes をコピーも
書き換えもせず、厳密な pending registry を追加します。

## 境界

- registry は既定で空であり、汎用 exemption list ではありません。
- pending entry は repository、完全な Work Item、provider PR、Contract base、canonical
  finalization head、registry append の親、正確な record path、3 つの正確な
  `In progress` row、created time を束縛します。
- 通常の archive、verification、finalization check が常に優先されます。
- defer できるのは欠けている 3 言語 parity row だけです。foreign、malformed、missing、
  mismatched、symlink、duplicate、stale、merged、partial、unrelated input は fail closed です。
- merge 後の documentation change は 3 row を原子的に追加して pending entry を削除し、
  predecessor history を変更しません。

## 検証

focused regression は有効な Git topology に加え、foreign、head/base/PR/path/row mismatch、
duplicate-key、missing record、symlink、unrelated append、partial row、default branch を
検証します。Manifest と route test は light、standard、strict の全 profile でこの regression
を必須にします。

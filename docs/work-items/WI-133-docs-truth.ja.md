---
author: AI Cockpit maintainers
workItemId: WI-133-docs-truth
title: Documentation truth の整合
description: merge 済み Work Item の文書と reference-parity の implementation baseline を現在の Runtime evidence に合わせる。
audience:
  - adopter
  - contributor
  - maintainer
status: implementation
authority: canonical
lastVerifiedBy: WI-133-docs-truth
---

# WI-133 — Documentation truth の整合

## Intent

完了して merge された Work Item を implementation のままにせず、三言語のページから
archived evidence と close decision へ安定して追跡できるようにします。

## Boundaries

- WI-130、WI-131、WI-132 の三言語ページを `implemented` にする。
- 各ページから archived verification evidence と close decision を参照できるようにする。
- reference-parity の current implementation baseline に正しい evidence path とともに三つを追加する。
- Runtime、Protocol bytes、過去の記録、release/version は変更しない。

## Acceptance

- すべての対応言語で documentation acceptance が通る。
- parity baseline と Work Item page の status と evidence path が一致する。
- 現在の implementation truth と歴史的な page 内容の境界が明示される。

## Verification

documentation acceptance と最終 diff review の結果は active Contract と Runtime evidence に記録します。

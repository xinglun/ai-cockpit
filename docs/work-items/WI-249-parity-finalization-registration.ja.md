---
author: AI Cockpit maintainers
title: "WI-249 — Parity finalization registration"
workItemId: WI-249-parity-finalization-registration
description: "parity を変更する Work Item に verification 前の lifecycle-bound terminal path 登録を要求します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-249-parity-finalization-registration
authority: canonical
---

# WI-249 — Parity finalization registration

WI-249 は immutable な WI-247 predecessor を recovery し、archive 後の documentation
mutation を必要とする順序 loop を取り除きます。自身の parity row は verification 前に
commit され、将来の archived Contract、verification、canonical finalize、close path を
列挙します。status は `In progress → verified close 後 Implemented` という明示的な条件式
です。PR #199 の review、merge、finalization、正確な cleanup、close より前に completion を
主張しません。

## Conditional control and quality profiles

governance integrity gate は active Contract の scope/acceptance と active Summary の changed
paths を検査します。それらが `docs/reference/reference-parity*` または parity registration を
所有すると明示した場合だけ、3 つの正確な lifecycle-bound row を要求します。static selector
は light profile で実行され、standard と strict が継承します。通常の non-parity code Work
Item は `active_non_parity` のままであり、広い profile によって documentation scope を課されません。

archived code 用 pending registry は独立した temporary bridge のままです。その
repository/PR/head/base/record binding、registry-only append topology、regular-file containment、
default-branch stale behavior は変更しません。

## Fail-closed evidence

regression は missing、partial、terminal-only、foreign-path、post-archive-only projection が
決定的に失敗することを証明します。有効な row は Git blame で導入 commit を特定し、Git
history で verification evidence の追加より厳密に前であることを証明します。同じ row bytes
が active、awaiting-merge-close、closed の各 state を通過し、archive evidence は書き換えません。

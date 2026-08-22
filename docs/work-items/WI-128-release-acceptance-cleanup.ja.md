---
author: AI Cockpit maintainers
workItemId: WI-128-release-acceptance-cleanup
title: Release adopter acceptance の cleanup と isolation truth
description: post-release acceptance の cleanup を fail closed にし、isolation receipt を監査可能に保つ。
audience:
  - maintainer
  - release-engineer
status: implemented
authority: canonical
lastVerifiedBy: release-acceptance
---

# WI-128 — Release acceptance cleanup

N-1 post-release harness は finish trap で明示的な exit-status variable を
一貫して扱う。upgrade と cleanup が成功した場合は必ず zero を返し、未設定の
status が有効な acceptance を shell error に変えることはない。

両 adopter harness は検証済み temporary root の cleanup、immutable な
`releasePublished` truth、cleanup receipt、監査可能な typed isolation manifest
を維持する。HOME/XDG configuration root は Runtime write forbidden、TMPDIR/
CARGO_HOME は隔離された allowed-write root として書き込みを記録する。

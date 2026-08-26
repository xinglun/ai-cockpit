---
author: AI Cockpit maintainers
title: "WI-299 — release adopter finalization の基線バインディング"
workItemId: WI-299-release-adopter-finalize-binding
description: "release adopter の finalization receipt をアーカイブ済み Work Item Contract の基線に固定します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-299-release-adopter-finalize-binding
terminalArchive: .ai/work-items/archive/WI-299-release-adopter-finalize-binding.contract.json
terminalVerification: .ai/evidence/WI-299-release-adopter-finalize-binding.verification.json
terminalFinalization: .ai/decisions/WI-299-release-adopter-finalize-binding.finalize.json
terminalDecision: .ai/decisions/WI-299-release-adopter-finalize-binding.close.json
authority: canonical
---

# WI-299 — Release adopter finalization の基線バインディング

## Intent

v0.2.32 staged adopter 受入で、実際の fail-closed 不一致が見つかりました。
スクリプトが変更後の HEAD を `pullRequest.baseRevision` に書いていましたが、
Runtime はアーカイブ済み Contract の基線を要求します。

## Scope

2 つの release adopter harness は、変更前に各 Work Item Contract の
`baseRevision` を読み取り検証します。finalization receipt は変更後の HEAD
を `headRevision` に保持し、`pullRequest.baseRevision` は保存した Contract
基線に固定します。staged と N-1 upgrade の静的回帰検査を追加します。

## Boundary

対象は harness と回帰テストの修正です。Runtime の lifecycle semantics、
v0.2.32 の過去 bytes、新しい adopter 技術スタックは変更しません。既存の
cleanup、isolation、immutable artifact、structured decision 検査は継続します。

## Verification

- adopter と upgrade の静的テストが成功すること。
- candidate 受入が `finalize-verify` と structured close まで到達すること。
- receipt が Contract の `baseRevision` と変更後 `headRevision` を区別すること。

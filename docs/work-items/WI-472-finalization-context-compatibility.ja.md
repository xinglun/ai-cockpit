---
author: AI Cockpit maintainers
title: "WI-472 — finalization context 互換性"
description: "finish と archive の前に pending provider sentinel を provisional として扱います。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-472-finalization-context-compatibility
status: implemented
authority: authorized
lastVerifiedBy: WI-472-finalization-context-compatibility
terminalArchive: .ai/work-items/archive/WI-472-finalization-context-compatibility.contract.json
terminalVerification: .ai/evidence/WI-472-finalization-context-compatibility.verification.json
terminalFinalization: .ai/decisions/WI-472-finalization-context-compatibility.finalize.json
terminalDecision: .ai/decisions/WI-472-finalization-context-compatibility.close.json
---

# WI-472 — finalization context 互換性

## Intent と境界

`pending:<stable-reference>` のような provider placeholder を完全な resource-finalization
plan と誤認しないようにします。review 済み provider resource が bind されるまでは Work Item
を復旧可能なままにします。本 Work Item は WI-471 や他の historical bytes を書き換えず、object
repository も操作しません。

## Scope

- `pending:*` と `unknown` の finalization context を provisional と判定します。
- 既存の `finish`/`archive` 境界で fail closed し、拒否時は active bytes を保持します。
- protocol と repository の regression test を追加し、3 言語の文書を同期します。

## Acceptance

1. pending provider context は `finish` または `archive` を通過できないこと。
2. 完全で review 済みの context は既存 lifecycle test を通過すること。
3. 拒否によって active Work Item bytes が移動・書換えされないこと。
4. 英語・簡体字中国語・日本語の test と文書が同じ provisional boundary を示すこと。
5. WI-471 は immutable のまま保持し、修正の release 後に explicit successor receipt でのみ recovery すること。

## Verification

- `cargo test --locked -p cockpit-protocol --test resource_finalization`
- `cargo test --locked -p cockpit-repository --test archive_integrity`
- `cargo test --locked --workspace`
- `python3 tests/conformance/reference_inventory_docs_test.py`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

provider PR がまだ不明な場合は、明示的な provisional context を使い Work Item を active に保ちます。
verification、finish、archive の前に正確な reviewed PR URL を bind し、immutable な archived Contract を
編集して pending sentinel を置き換えないでください。

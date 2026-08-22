---
author: AI Cockpit maintainers
title: "WI-114 Release adopter ライフサイクル順序"
description: "公開 Release と N-1 acceptance が Runtime のライフサイクル契約に従うようにする。"
audience:
  - maintainer
  - reviewer
  - release_operator
status: implemented
authority: canonical
lastVerifiedBy: v0.2.8-adopter-acceptance
capabilityClaims:
  - release_adopter_acceptance
  - fail_closed_lifecycle
---

# WI-114：Release adopter ライフサイクル順序

## 目的

公開 adopter と N-1 acceptance harness が fail-closed のライフサイクル契約に
従い、`checkpoint` より前に `preflight` を記録するように修正する。

## この Work Item が必要な理由

immutable な v0.2.8 Release により harness の不具合が判明した。両スクリプトが
`start → checkpoint → preflight` を実行していたため、v0.2.8 Runtime は正しく
拒否した。本 Work Item は acceptance harness と回帰チェックだけを変更し、公開
済み Release とその receipt は書き換えない。

## Acceptance

- 公開 adopter acceptance は `lifecycle-preflight` を
  `lifecycle-checkpoint` より先に記録する。
- N-1 acceptance は `old-preflight` を `old-checkpoint` より先に記録する。
- N-1 acceptance は旧 Runtime で旧 Work Item を close して historical evidence を保持し、
  migration 後に新しい Work Item を作り、v0.2.8 で `new-preflight` → `new-checkpoint` →
  `new-verify` を記録する。
- どちらかの順序が戻った場合、static test が失敗する。
- 両 harness は immutable な公開 artifact のみを使い、cleanup、isolation、
  checksum、`first-adopter-smoke=not_ready` の検証を保持する。
- 公開 v0.2.8 に対する再実行が source/workspace fallback なしで成功する。
- N-1 harness は旧 summary の lifecycle state を偽造しない。migration 前に旧 lifecycle を
  close し、migration 後は新 Work Item の lifecycle を使用する。

## 検証

```text
bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance_test.sh
AI_COCKPIT_RUN_PUBLIC_ACCEPTANCE=1 AI_COCKPIT_ACCEPTANCE_TARGET=aarch64-apple-darwin \
  bash tests/release/adopter_acceptance_test.sh
bash tests/release/adopter_upgrade_acceptance.sh \
  --repository xinglun/ai-cockpit --from-tag v0.2.7 --to-tag v0.2.8 \
  --target aarch64-apple-darwin --output ./release-adopter-upgrade-acceptance
```

公開済み v0.2.8 Release は immutable のまま保つ。失敗した receipt は失敗の
post-release evidence として保存し、Release 成功の根拠には使用しない。
修正後の public adopter と N-1 run はいずれも cleanup receipt 付きで成功した。
N-1 は公開 v0.2.7 → v0.2.8 pair を使い、migration 後の Work Item を開始する前に
旧 evidence bytes を保持した。

## Outcome

状態：**acceptance harness の修正。Release の truth は immutable のまま。**

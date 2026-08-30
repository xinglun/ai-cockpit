---
author: AI Cockpit maintainers
title: "WI-431 — v0.2.47 release recovery"
description: 不変の v0.2.46 tag を移動せずに公開を回復する。
audience: [adopter, maintainer, reviewer]
status: implemented
authority: human-authorized
workItemId: WI-431-release-v0-2-47-recovery
lastVerifiedBy: WI-431-release-v0-2-47-recovery
terminalArchive: .ai/work-items/archive/WI-431-release-v0-2-47-recovery.contract.json
terminalVerification: .ai/evidence/WI-431-release-v0-2-47-recovery.verification.json
terminalFinalization: .ai/decisions/WI-431-release-v0-2-47-recovery.finalize.json
terminalDecision: .ai/decisions/WI-431-release-v0-2-47-recovery.close.json
---

# WI-431 — v0.2.47 release recovery

## Intent と境界

最初の v0.2.46 公開試行は、closed Work Item のドキュメント promotion が
tag 前に完了していなかったため release source gate に拒否されました。
この tag は不変の失敗履歴として保持し、移動も再ラベルも行いません。この
Work Item では terminal ドキュメントを promotion してから新しい patch
Release を作成し、公開 artifact 経路を端から検証します。

これは release/documentation recovery であり、Runtime source、CI workflow
policy、Repository Protocol は変更しません。

## Acceptance

- tag 作成前に三言語の closed Work Item documentation が promotion 済みである。
- Cargo metadata と lockfile を v0.2.46 から v0.2.47 へ一つの patch として
  進め、失敗した v0.2.46 tag を再利用しない。
- 公開 v0.2.47 artifact が manifest、checksum、SBOM、provenance、platform
  smoke、adopter、N-1 acceptance を downloaded artifact だけで通過する。
- v0.2.46 の失敗が unpublished immutable tag として記録される。
- reviewed merge、finalization、close、同期、正確な cleanup 後に default branch
  が `ready_on_base` になる。

## Verification 境界

Release route は strict gate manifest、workspace tests、三言語 documentation
check、公開 release harness を実行します。post-release receipt は downloaded
binary、release manifest、checksum、Runtime identity、adopter repository
identity、isolation manifest、cleanup result を bind しなければなりません。

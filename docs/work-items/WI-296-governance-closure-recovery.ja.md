---
author: AI Cockpit maintainers
title: "WI-296 — Governance closure recovery"
workItemId: WI-296-governance-closure-recovery
description: "Consumed retry の履歴投影と parity、terminal finalization の整合性を回復します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-296-governance-closure-recovery
authority: canonical
---

# WI-296 — Governance closure recovery

## Intent

Work Item の close 後も consumed retry を履歴 evidence として残し、文書と
finalization gate を Runtime が実際に生成する terminal receipt に合わせます。

## Scope

- Consumed retry の履歴を current error ではなく historical として保持する。
- merge と正確な cleanup が一つの観測で確定した場合、identity-bound な direct
  terminal finalization receipt を受け入れる。
- partial、malformed、foreign、forked evidence は引き続き fail-closed にする。
- immutable closure evidence に基づき WI-294 の terminal 文書と三言語 parity ledger
  を同期する。

## Boundary

Rust Core の挙動、release/adopter harness、historical archive bytes は対象外です。

## Acceptance

- confirmed close 後も consumed retry が historical として表示される。
- direct terminal receipt は merged/deleted state と merge identity が揃う場合だけ受理し、
  transition chain は sequence 1 と 2 を引き続き要求する。
- WI-294 文書が immutable closure evidence から正しく昇格される。
- repository gate 全体と hosted checks が成功する。

## Verification

close 前に、installed Runtime lifecycle、repository governance gate、documentation
acceptance、hosted quality checks を実行します。

## Unknowns

Work Item owner が明示するまで user-visible benefit は unknown のままです。

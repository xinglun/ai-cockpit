---
author: AI Cockpit maintainers
title: "WI-245 — documentation status と parity recovery"
workItemId: WI-245-doc-status-parity-recovery
description: "current main 上で WI-240 を回復し、stale conditional documentation を terminal repository evidence に bind する。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-245 — documentation status と parity recovery

WI-245 は immutable failed delivery WI-240 の Runtime-recorded successor でしたが、
repository gate ID が global order でなかったため hosted quality で失敗しました。
`.ai/decisions/WI-245-doc-status-parity-recovery.recovery.json` はこの immutable failure
を保持し、current redelivery を WI-252 に割り当てます。

## Acceptance boundary

- deterministic reference inventory は pinned Git tree から導出し、dirty/untracked checkout
  metadata を除外して、720 deferred records と正確に 4 capability/profile `migrate-gap` を保持します。
- tri-language Work Item status は authoritative archived Contract と close/recovery evidence に
  対して検査します。closed Work Item に conditional/after-close parity prose が残れば失敗します。
- WI-241、WI-249、WI-251 の terminal row は archived Contract、verification evidence、canonical
  finalization、sequence-2 deleted cleanup transition、structured close decision を bind します。
- provider truth が `immutable: false` のため、v0.2.31 は identity-bound かつ drift-detectable です。
  durable adopter baseline は `aarch64-apple-darwin`、hosted Linux run `32696048024` は
  provider-retained external evidence です。

## Verification と lifecycle

WI-245 の immutable archive は successor delivery の外にあるため、この branch では
WI-245 を current parity Work Item として登録しません。WI-252 は適用可能な implementation
と documentation change のみを replay し、独自の Runtime verification、hosted review、
structured close を完了する必要があります。

## References

- [WI-240 predecessor](WI-240-doc-status-consistency.ja.md)
- [Reference file comparison](../reference/reference-file-comparison.ja.md)
- [Reference source parity](../reference/reference-parity.ja.md)
- [Release distribution](../release/distribution.ja.md)

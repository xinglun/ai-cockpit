---
author: AI Cockpit maintainers
title: "WI-252 — Manifest gate-order recovery"
workItemId: WI-252-manifest-gate-order-recovery
description: "immutable failed WI-245 delivery を recovery し、repository gate ID を globally sorted かつ unique にする。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-252-manifest-gate-order-recovery
authority: canonical
---

# WI-252 — Manifest gate-order recovery

WI-252 は immutable failed delivery WI-245 の Runtime-recorded successor です。
predecessor recovery receipt は WI-245 の Contract、Summary、Outcome、Events、
archive、verification、finalization digest を binding します。これらの historical
bytes は本 delivery の外に保持され、rewrite されません。

## Acceptance boundary

- `tests/ci/repository_gate_manifest.json` の gate ID は global lexical order かつ
  unique です。したがって `docs_pending_parity_registry_regression` は
  `docs_work_item_status_consistency` より前にあります。
- duplicate と out-of-order fixture manifest は、hosted quality と同じ validation
  により route selection 前に fail closed します。
- WI-245 のうち現在も適用可能な documentation status、inventory、release truth の
  change だけを `origin/main@87bfd866` に replay します。predecessor archive が存在
  しない entry を current parity Work Item として誤登録しません。
- pinned comparison は 720 deferred entry と正確に 4 capability/profile
  `migrate-gap` を保持します。provider truth は immutable と claim せず、
  identity-bound かつ drift-detectable のままです。

## Verification と lifecycle

regression は最初に PR #203 の `gate IDs must be deterministic` failure を再現し、
ID の ordering と negative fixture の追加後に manifest / quality-route suites を
通過しました。full docs、governance、format、clippy、workspace、installed Runtime、
exact-head hosted check も必要です。この pre-archive row は future archived Contract、
verification evidence、canonical finalization、structured close を参照し、reviewed
close 前に完了を claim しません。

## References

- [WI-245 failed predecessor](WI-245-doc-status-parity-recovery.ja.md)
- [Reference source parity](../reference/reference-parity.ja.md)
- [Reference file comparison](../reference/reference-file-comparison.ja.md)
- [Release distribution](../release/distribution.ja.md)

---
author: AI Cockpit maintainers
title: "WI-237 — release route recovery と v0.2.31 公開"
workItemId: WI-237-release-route-recovery-v0-2-31
description: "clean-batch の release route を修正し、v0.2.30 の履歴を書き換えずに次の immutable patch release を公開します。"
audience:
  - maintainer
  - reviewer
  - adopter
status: recovered
authority: canonical
lastVerifiedBy: WI-237-release-route-recovery-v0-2-31
---

# WI-237 — release route recovery と v0.2.31 公開

この Work Item は clean-batch boundary で発見された release quality route の欠陥を修正します。
active Work Item directory が存在しない状態は正当ですが、release workflow の Contract discovery
で失敗してはいけません。immutable な v0.2.30 tag と公開失敗の事実は履歴として保持し、書き換えません。
修正済み route で v0.2.31 を公開し、公開後の adopter と N-1 acceptance は successor Work Item に委譲します。

## Acceptance boundary

- `.ai/work-items/active` が存在しない場合も release route planning は決定的に成功し、zero-active
  Work Item の regression を持つ。
- package metadata、lockfile、release 文書、三言語 parity は v0.2.31 を示し、v0.2.30 は失敗した
  immutable history として保持する。
- hosted release checks は immutable v0.2.31 artifact だけを公開する。
- v0.2.31 の public artifact identity、installed Runtime check、isolated adopter/upgrade acceptance
  は successor Work Item が担当する。

## References

- [Release and Distribution](../release/distribution.ja.md)
- [Versioning](../architecture/versioning.ja.md)
- [Reference parity ledger](../reference/reference-parity.ja.md)

---
author: AI Cockpit maintainers
title: "WI-238 — v0.2.31 release recovery"
workItemId: WI-238-release-v0-2-31-recovery
description: "不変な WI-237 recovery の後、clean な default branch から v0.2.31 を再配信する。"
audience:
  - maintainer
  - reviewer
  - adopter
status: implemented
authority: canonical
lastVerifiedBy: WI-238-release-v0-2-31-recovery
---

# WI-238 — v0.2.31 release recovery

WI-238 は WI-237 の clean successor です。WI-237 の不変 archive と pre-merge
finalization receipt は、hosted quality が三言語 parity binding の不足を検出し、
修正試行が未 merge の head を進めたため、そのまま保持します。本 Work Item は
同期済み default branch から同じ範囲の release 修正を再配信します。

## Acceptance boundary

- active Work Item directory がない repository でも release quality route が決定的
  に通り、regression test がある。
- hosted checks の前に三言語 parity row が verification evidence と pre-merge
  finalization receipt を bind する。
- 失敗した不変 v0.2.30 tag は書き換えず再利用しない。v0.2.31 は reviewed merge
  head の hosted checks 成功後だけ公開する。
- 公開 v0.2.31 と N-1 upgrade acceptance は immutable な download artifact のみを
  使用し、隔離 root と cleaned temporary run root を証明する。

## Recovery boundary

WI-237 は不変な historical recovery evidence として保持し、successor は
`.ai/decisions/WI-237-release-route-recovery-v0-2-31.recovery.json` によって bind
します。predecessor の Contract、Summary、Outcome、Events、verification、archive、
finalization receipt は書き換えません。

## References

- [Reference parity ledger](../reference/reference-parity.ja.md)
- [WI-237 immutable Work Item](WI-237-release-route-recovery-v0-2-31.ja.md)
- [Release distribution](../release/distribution.ja.md)

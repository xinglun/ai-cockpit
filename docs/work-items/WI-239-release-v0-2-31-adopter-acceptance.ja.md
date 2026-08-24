---
author: AI Cockpit maintainers
title: "WI-239 — v0.2.31 公開 Release adopter 受入れ"
workItemId: WI-239-release-v0-2-31-adopter-acceptance
description: "隔離した新規 adopter で不変な v0.2.31 Release を受入れ、インストール済み Runtime を本リポジトリへバインドする。"
audience:
  - maintainer
  - reviewer
  - adopter
status: current
lastVerifiedBy: WI-239-release-v0-2-31-adopter-acceptance
authority: canonical
---

# WI-239 — v0.2.31 公開 Release adopter 受入れ

本 Work Item は v0.2.31 の post-release 受入れ境界である。Runtime 操作には
公開 Release archive だけを使い、source checkout や workspace binary を
Runtime fallback として使わない。

## 受入れ境界

- 公開 tag が draft/prerelease ではなく、archive、manifest、checksums が不変な
  Release identity に一致する。
- HOME、XDG_CONFIG_HOME、TMPDIR、CARGO_HOME を隔離した新規 adopter を作成し、
  attach、profile 確認、Agent doctor、repository identity、isolation を通す。
- `first-adopter-smoke` は `not_ready` のままにし、intent、scope、受入れ条件、
  authority、approval、completion を推測で埋めない。
- 2 回目の verify は証拠を再利用し、プロセスを再起動しない。lifecycle は構造化
  human decision receipt 付きで close まで完了する。
- 成功時に一時 acceptance run root を検証して削除する。
- インストール済み v0.2.31 binary で本リポジトリに対する明示的な repository-bound
  inspect、status、doctor、Agent doctor を通す。

## Evidence

受入れ receipt、Runtime identity、Release manifest、隔離 manifest、証拠再利用出力、
lifecycle evidence、インストール Runtime の検査結果は
`.ai/evidence/WI-239-release-v0-2-31-adopter-acceptance/` に保存する。

## 参照

- [Release distribution](../release/distribution.ja.md)
- [Outcome report](../reference/outcome-report.ja.md)
- [Agent workflow](../reference/agent-workflow.ja.md)

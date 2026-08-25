---
author: AI Cockpit maintainers
title: "WI-260 — Recovery-aware governance gate"
workItemId: WI-260-recovery-gate
description: "不変 predecessor の recovery を governance inventory と文書 promotion で収束させる。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-260-recovery-gate
authority: canonical
---

# WI-260 — Recovery-aware governance gate

## Intent

有効な recovery receipt を持つ不変 predecessor は、歴史的 close が非規範でも
`recovered` として投影し、通常の文書 promotion は approved close に限定します。

## Scope

この Work Item は governance-integrity inventory、closed Work Item の文書
promotion helper、および回帰テストを更新します。Runtime lifecycle は変更せず、
WI-258 の歴史的 close bytes は書き換えません。

## Acceptance

- 有効な recovery と無効な歴史 close の組合せは `recovered` となり、
  `invalid_terminal_decision` を出しません。
- 有効な approved close は古い recovery より優先されます。
- 文書 promotion は有効な recovered predecessor だけをスキップし、無効な recovery は
  fail closed します。
- retry recovery は `successorWorkItemId` を省略できますが、successor/supersede の decision は
  明示的な successor binding を引き続き要求します。
- 曖昧でない短縮 Git revision は一意の commit に解決して finalization binding に使い、曖昧または無効な
  revision は fail closed のままです。
- gate と promotion の双方に回帰テストがあります。
- 三言語 Work Item と parity 行が修正 evidence を束縛します。

## Evidence boundary

Recovery は履歴上の terminal projection であり、green completion の主張ではありません。
将来の実装 promotion は successor Work Item が所有し、predecessor の元 bytes は不変・監査可能です。

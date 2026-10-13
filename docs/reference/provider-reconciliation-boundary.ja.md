---
author: AI Cockpit maintainers
title: "Provider reconciliation の境界"
description: "過去の provider inventory は文脈であり、現在の provider truth ではない。"
audience: [maintainer, reviewer]
status: implemented
authority: canonical
lastVerifiedBy: WI-348-reference-verification-operation-policy
---

# Provider reconciliation の境界

[English](provider-reconciliation-boundary.md) · [简体中文](provider-reconciliation-boundary.zh-CN.md)

`open-pr-issue-reconciliation-662.*` と `pre-release-documentation-alignment.json` は、
source repository の過去の revision における provider/reviewer assessment です。過去に
観測された状態を記録しますが、現在の repository、GitHub PR、release、enterprise approval
を証明しません。

AI Cockpit は provider の責任を明確に分けます。

- Runtime は delegated evidence を require、bind、display、archive できます。
- GitHub/Hosted CI、reviewer、branch protection、release publication、enterprise retention は外部です。
- stale または欠落した reconciliation は unknown であり、merge/release/close を許可しません。
- 新しい provider observation は現在の repository と Work Item identity で再取得し、digest、timestamp、source を持ちます。

したがって file-by-file ledger では source JSON/Markdown を `reference-only` とします。
`.ai/` にコピーせず current status に混ぜず、repository-local Contract や Runtime evidence を上書きしません。
target の境界は [Release distribution](../release/distribution.ja.md) と [Reference parity](reference-parity.ja.md) を参照してください。

## 明示的なライフサイクル配置の証拠

Schema-v3 の resource finalization は `postMergeSync.lifecycleLayout`（`separate` または `combined`）と `postMergeSync.documentation`（`projected` または `absent`）を使用します。schema-v1/v2 の規則は維持します。`combined` は実際の parent→commit の finish/archive 遷移を束縛し、中間 commit を作りません。Runtime は正規の Summary、Outcome、report、events、archive manifest、verification を一緒に検証します。凍結 report の履歴 active パスは有効です。既存 verification と失敗 attempt は元のバイト列と Runtime identity を保持し、新規 attempt は実行証拠に束縛されます。

元の PR identity は不変です。sync PR は実際の base、head、merge に束縛され、base は source merge を含みます。Runtime は競合のない Git merge 結果を証明し、競合や merge への追加内容を拒否します。merge tree と sync head tree が同一とは仮定しません。sync branch の変更は当該 WI の lifecycle artifact と必要な文書に限定します。

`absent` は文書免除ではなく明示的な証拠です。source head、lifecycle parent、archive、sync base/head、merge に WI ページ、parity 行、文書の active 参照がなく、Contract も欠席を許容する必要があります。履歴 evidence/report の参照は有効です。provider identity、新鮮度、曖昧性の確認、同じ Git common directory の残存 control checkout、Retained → Deleted、finalize-verify、独立した実際の Human close Decision を引き続き必須とします。

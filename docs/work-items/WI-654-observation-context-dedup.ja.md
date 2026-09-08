---
author: AI Cockpit maintainers
title: WI-654 — 観察コンテキストの重複排除(調査)
description: preflightにおける実測された重複読み取りの発見と、一見安全に見えた修正が実は安全ではなかった理由。
workItemId: WI-654-observation-context-dedup
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-654-observation-context-dedup
terminalArchive: .ai/work-items/archive/WI-654-observation-context-dedup.contract.json
terminalVerification: .ai/evidence/WI-654-observation-context-dedup.verification.json
terminalFinalization: .ai/decisions/WI-654-observation-context-dedup.finalize.json
terminalDecision: .ai/decisions/WI-654-observation-context-dedup.close.json
---

# WI-654 — 観察コンテキストの重複排除(調査)

本 Work Item はアーキテクチャ最適化専項の P1-B にあたり、P0 マップ
（`docs/reference/architecture-responsibility-map-2026-09.md`）が特定した
最も具体的な重複読み取りの例に絞って調査した。本番コードの変更は一切
出荷していない ── 一見明らかに安全に見えた修正が、実は安全ではないことが
判明し、本文書がその発見内容である。

## 計測

一時的な、コミットしない計測用の instrumentation（`repository_id` と
`snapshot_digest` をラップするアトミックカウンタ、コミット前に必ず
リバート ── WI-648/649/650 と同じ手法）を追加し、1つの active Work Item を
持つ、新規に attach したフィクスチャリポジトリに対して実行した:

| コマンド | `repository_id` 呼び出し回数 | `snapshot_digest` 呼び出し回数 |
|---|---|---|
| `preflight` | 3 | 2 |
| `checkpoint` | 3 | 2 |
| `status`(active Work Item あり) | 1 | 0 |

これにより、`preflight`/`checkpoint` に限っては、単なる理論上の懸念ではなく
実際に request-scoped の重複計算が発生していることを確認した。

## 仮説と、それが偽である理由

3回の `repository_id` 呼び出しのうち1回は `project_governance_unknowns`
（`crates/cockpit-repository/src/project_governance.rs:352`）内にあり、
`.ai/project/capabilities.json` を検証するための期待 identity を束縛するために
`repository_id(&root)` を呼んでいる。仮説は次の通りだった:
`contract_freshness_findings`（`crates/cockpit-repository/src/lib.rs:9330`）が
同じガバナンス判定（`governance_decision_for_contract_base_internal_with_archive`
は 9434行目で `contract_freshness_findings` を、9461行目で
`project_governance_unknowns` を呼ぶ）の中でより早く実行され、既に
`contract.repository_id != repository_id(&root)` をチェックしているので、
`project_governance_unknowns` が実行される時点では `contract.repository_id` は
既に fresh な値と等しいはずであり、それを再利用すれば `.ai/cockpit.toml` への
冗長なディスク読み取りを、挙動を変えずに除去できるはずだ、というものだった。

実際のソースコードでこれを検証すると、この仮説は偽であることがわかった:

```rust
// crates/cockpit-repository/src/lib.rs:9330
if contract.repository_id != repository_id(&root).to_string() {
    findings.push("stale_contract".into());
}
```

`contract_freshness_findings` は不一致の際に `stale_contract` という finding
を**記録するだけ**であり、早期 `return` はせず、
`governance_decision_for_contract_base_internal_with_archive` がその後
`project_governance_unknowns` を呼び続けることを止めない。したがって、
foreign または stale な Contract（`repository_id` フィールドが実際の
リポジトリともはや一致しないもの）でも、未検証で誤っている可能性のある
`contract.repository_id` を伴ったまま `project_governance_unknowns` に
到達しうる。これを `repository_id(&root)` の代わりに使うと、まさにこの
エッジケースで `load_declaration` の `expected_repository_id` 比較が変わり、
現在のコードとは異なる `project_capabilities_repository_mismatch` 系の
unknown を生成する可能性がある ── これは大半のテストが検証する
（fresh で一致する）通常ケースでは見えないものの、純粋なリファクタリングでは
なく実際の挙動差異である。

この置き換えを実装した試行版パッチは、この検証の後に破棄した。本専項自身の
「互換性リスクを実際に確認していない変更を出荷しない」というルールに従った
ものである。

## 安全な修正に必要なこと

この呼び出しを安全に除去するには、一度だけ解決した `repository_id` の値を
`contract_freshness_findings`、`governance_decision_for_contract_base_
internal_with_archive`、`project_governance_unknowns` の3関数に通す必要がある。
`contract_freshness_findings` は `pub fn` であり `lib.rs` に2つの呼び出し箇所
（8900、9434）がある。ガバナンス判定関数群は、P0マップの通り、それぞれ
2〜4個のほぼ重複した `_internal`/`_with_archive`/`_with_runtime` バリアントを
持ち、本 Work Item が計測した preflight パスをはるかに超えて使われている。
このシグネチャ群を正しく変更し、全呼び出し箇所で挙動が変わらないことを検証し、
テストで証明することは、本 Work Item が実測で正当化できる範囲（小さな
ローカル TOML の読み取り1回の除去）をはるかに超える改修である。これは
将来の Work Item に委ね、より強い実測上の必要性か、同じ関数群への他の
必須変更との合流のいずれかを条件とする。

2つ目に計測された重複（`snapshot_digest` が、同一の、既にメモリ上にある
`RepositorySnapshot` 値に対して、`project_governance_unknowns` 内で1回、
`apply_preflight_review_evidence` 内で1回、計2回呼ばれている）は、両方の
呼び出しが証明可能に同じ入力に対して行われているという点で、原理的には
よりクリーンな候補である ── しかし、これを通すにも上記と同じシグネチャ変更の
範囲が必要であり、除去対象のコスト（作業ツリーがダーティな場合にのみ
`git` サブプロセスを起動する、プロセス内のダイジェスト計算）が有意であると
計測されていない。同じ理由でここでは見送る。

## P0 マップへの追記(先送り)

`docs/reference/architecture-responsibility-map-2026-09.md`（WI-652）は
このパターンを重複読み取りの例として挙げていたが、特定の修正が安全かどうかは
評価していなかった。本 Work Item ではこの文書を修正しない。このブランチには
その文書が存在しないためである（WI-652 はまだマージされていない）。WI-652が
マージされた後、本 Work Item のより具体的な発見（明白に見えた修正が実は
安全ではなく、その理由）を追記し、将来の読者が同じ反証済みの置き換えを
再度試みないようにすべきである。

## 検証

本 Work Item の最終状態では本番コードを一切変更していない。`cargo fmt`、
`cargo clippy --all-targets --all-features -- -D warnings`、
`cargo test --locked --workspace` は変更していないワークスペースに対して
成功する。

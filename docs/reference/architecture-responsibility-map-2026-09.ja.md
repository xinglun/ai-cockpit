---
author: AI Cockpit maintainers
title: アーキテクチャ責任・依存関係マップ (2026-09)
description: 観察・治理・生命周期・証拠・実行・投影の各事実の現在の所有者、およびP1〜P3アーキテクチャWork Itemの事実的根拠。
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-652-architecture-responsibility-map
---

# アーキテクチャ責任・依存関係マップ (2026-09)

本文書は 2026-09 アーキテクチャ最適化専項の P0 成果物である。目標設計ではなく
現在の責任境界を文書化するものであり、後続の各 Work Item（P1-B 観察コンテキスト、
P2-A 生命周期/ストレージ抽出、P2-B 状態型、P2-C マルチファイル一貫性、P3 物理実行
境界）が同じ引用済みの事実的根拠を共有し、再導出せずに済むようにする。本 Work Item
はコードを一切変更しない。

North Star: Calibrated Human-Agent Trust。現在どこで責任が混在しているかを
文書化することは、現状の挙動が誤っているという主張ではない。これは、ガバナンス上の
真実を変えずに修正結合度を下げるための前提条件である。

## モノリスの規模

`crates/cockpit-repository/src/lib.rs` は 18220 行 ── ワークスペース内で
圧倒的に最大のファイルである。同じ crate には `governance_controls.rs`
（1753行）、`outcome_render.rs`（639行、WI-653で純化済み、本文書では再検討しない）、
`project_governance.rs`（402行）もある。

## lib.rs の機能クラスタと現在の I/O/判定の混在状況

| クラスタ | おおよその行範囲 | 1関数内でI/Oと判定が混在しているか |
|---|---|---|
| リポジトリ identity (`repository_id`、`new_repository_id`) | 728-763 | いいえ ── 純粋な導出/読取 |
| 検証 reuse 判定 (`assess_verification_reuse*`、実行可能ファイル identity ハッシュ) | 763-1658 | はい ── 例えば `assess_verification_reuse_measured`(777) は identity 事実の収集と reuse 判定の両方を行う |
| Receipt store / capability-scoped nofollow ファイルI/O | 1660-2832 | いいえ ── 自己完結したストレージプリミティブ層で、既にガバナンスロジックから分離されている |
| Attach / migration | 2832-3337 | はい ── ファイルI/OとプロトコルバージョンPmの判定が関数ごとに混在 |
| Status / readiness (`status_with_runtime` 3343、`repository_readiness*` 3426-3542、`historical_finalization_inventory` 3832) | 3337-4209 | はい ── 各関数がファイル/gitを読み、readiness判定をインラインで計算（このクラスタにWI-648/649のボトルネックがあった） |
| Lifecycle start/checkpoint/preflight (`preflight_work_item_internal` 5107 他) | 4209-5293 | はい ── preflight は Contract+Summary+snapshot を読み、ガバナンス判定ヘルパーを呼び、preflight 判定を1関数内で書き込む |
| Finish (`finish_work_item_internal` 5320) | 5293-5670 | はい ── 同様の読取+判定+書込の混在 |
| Recovery / revalidation | 5670-6846, 16162-16451 | はい ── predecessor/successor 紐付け検証と recovery 判定記録が混在 |
| Evidence retention / audit export | 7470-8102 | 部分的 ── policy駆動の読取+書込で、比較的自己完結 |
| Delegated (外部) evidence import/list | 8102-8328 | いいえ ── ファイル束縛、自己完結 |
| Snapshot digest化 / policy解決 (`source_tree_digest`、`snapshot_digest` 8548、`policy_document`、`effective_policy_for_contract`、`resolve_verification_route`、`evaluate_contract_quality_gate`) | 8328-9016 | この層ではいいえ ── 既にキャプチャ済みのsnapshotに対する読み取り専用で、実質的なガバナンス判定の中核 |
| ガバナンス判定エントリポイント (`governance_decision_for_contract*` 他) | 9349-10389 | 分岐は多いが集約されている ── `_internal`/`_with_archive`/`_with_runtime` のほぼ重複したバリアントが各2〜4個 |
| Archive (`archive_work_item_internal` 10554) | 10389-11022 | はい ── finish と同様の読取+判定+書込の混在 |
| Resource finalization (plan/record/verify/resolve-head) | 11022-13152 | はい ── ファイルI/O+検証+チェーン解決が混在（WI-648/649の領域。`resolve_resource_finalization_head_with_candidates` は候補収集とチェーン解決を既に分離） |
| Close (`close_work_item_with_structured_decision_internal` 13242、約300行) | 13152-13615 | はい ── Contract/Summary/archiveを読み、policyを検証し、resource finalizationを検証し、決定ファイルを書く、が全てインライン |
| Knowledge / capability truth / performance diagnosis / work-item intelligence | 13906-16784 | いいえ ── 大部分は読み取り専用の投影 |
| Parallel slot leasing | 17001-17322 | いいえ ── ファイルリースによる相互排他、自己完結 |
| Compatibility/scope relations | 17402-17720 | いいえ ── 純粋関数、I/Oなし、よく分離されている |
| 共有低レベルヘルパー (`read_json`、`atomic_json`、`atomic_write`、`observe`/`observe_cached` 17848/17957、`collect_files`) | 17770-末尾 | いいえ ── ただし `observe`/`observe_cached` は汎用のスナップショット観察エントリポイントであり、他の生命周期関数はこれを呼ぶ代わりに独自に再導出することが多い |

## 具体的な重複・責任混在の例

- **`repository_id` 導出の重複**: lib.rs、`governance_controls.rs`、
  `project_governance.rs` を横断するほぼ全ての呼び出し箇所で `.ai/cockpit.toml`
  から再計算/再読込されている（例: `project_governance.rs:306,352,398` は
  それぞれ独立に `crate::repository_id` を呼んでおり、request-scoped context を
  通じて解決済みの値を1つ受け取る形にはなっていない）。
- **`snapshot_digest` 計算の重複**: `lib.rs:8548` で定義されているが、
  `project_governance.rs:305,349,395` はそれぞれ、呼び出し元が同一リクエストの
  fresh な snapshot を既に保持している場合でも、declaration 読み込み時に独立して
  再計算している。
- **読取+判定+永続化が1関数に混在**: `preflight_work_item_internal`(5107)、
  `finish_work_item_internal`(5320)、`archive_work_item_internal`(10554)、
  `close_work_item_with_structured_decision_internal`(13242) はいずれも
  Contract/Summary/snapshot ファイルを読み、複数のガバナンス判定ヘルパーを呼び、
  結果の決定/outcome/archive アーティファクトを書き込む ── 関数本体内で
  「事実収集」「判定」「永続化」の各フェーズが分離されていない。
- **書き込みを行う「validator」**: `governance_controls.rs:1190
  record_work_item_governance_controls` は、モジュール自身のヘッダーコメントが
  シナリオや最終次元のevidenceを生成しないと説明しているにもかかわらず、
  書き込みを行っている ── モジュールが標榜する読み取り専用validator境界が、
  中の全関数で完全には守られていない。
- **依存の方向**: 循環依存は見つからなかった。`governance_controls.rs` と
  `project_governance.rs` は共有プリミティブ（`ObserverError`、`repository_id`、
  `snapshot_digest`、`reject_duplicate_json_keys`）のためだけに lib.rs を
  呼び戻しており、lib.rs は両モジュールを前方に呼び出している。これは
  一方向のレイヤリング（lib.rs → 両モジュール）であり循環ではないが、両モジュールは
  lib.rs のプリミティブに依存しているため、lib.rs が共有基盤であり続けない限り
  現状は独立した crate として切り出せないことを意味する。

## 既に良く分離され再利用可能なもの

- **`RepositorySnapshot`**（`cockpit-git/src/lib.rs:212`）と
  `GitRepository::snapshot()`(268) は、既に下位関数（`repository_readiness_from_snapshot`、
  `project_governance_projection`、`source_tree_digest`、`snapshot_digest`）に
  パラメータとして通されている、クリーンで単一の事実キャプチャ地点である。
  P1-B が構築すべき正しい抽象であり、単に今日、全エントリポイントから全リーフ
  関数まで一貫して通されていないだけである。
- **`RuntimeContext`**（`cockpit-protocol/src/lib.rs:120`）は、クリーンで
  最小限の identity 構造体であり、全ての `_with_runtime` バリアントを通じて
  一貫して `&RuntimeContext` として渡されている。
- **純粋な validator**: `governance_controls.rs` の `required_verification_checks`、
  `validate_checkpoint_evidence_bindings`、lib.rs の
  `scope_pattern_relation`/`work_item_compatibility`(17450-17720) は既に、
  型付きの事実を入力として受け取り、I/Oや副作用なしに判定を出力している ──
  P2-B の状態型作業はこれに倣うべきテンプレートである。
- **capability-scoped nofollow ファイル層**(2313-2832) は、既にガバナンス
  ロジックから分離された、再利用可能で自己完結したストレージプリミティブである ──
  P2-A の evidence/ストレージ層はこれを再利用すべきであり、再構築すべきではない。
- **`OutcomeRenderInput`/`outcome_render_input(_with_runtime)`**（WI-653、
  `outcome_render.rs`）は、本専項が一般化を望む
  「一度だけ観察/組み立て/描画」の分離を体現した最初の具体例である。

## 物理実行とガバナンス判定 (P3の事実的根拠)

`PhysicalSingleFlightCoordinator` は `crates/cockpit-verification/src/lib.rs:1473`
に存在し、これはガバナンス判定コード（`governance_decision_for_contract*`、
`require_green_governance*`、`evaluate_contract_quality_gate`）が存在する
`cockpit-repository` とは全く別の crate である。物理実行とガバナンス判定の
crateレベルでの分離は既に存在しており、P3 がこれを新設する必要はない。
まだきれいに分離されていないのは: `assess_verification_reuse*`（「この receipt は
再利用可能か」という identity 一致判定）が、それが判定対象とする
coordinator/executor と同じ `cockpit-verification` ではなく
`cockpit-repository::lib.rs:763` に存在している点である ── reuse 判定と
物理実行/合流の仕組みが、明確な単一の境界所有者なしに2つの crate にまたがっている。
このずれは本 Work Item ではこれ以上調査していない。実行共有を拡大するか
決める前に P3 が解決すべき具体的な未解決課題である。

## 候補となるリファクタリング: 問題・目標境界・互換性リスク・検証方法

### P1-B — 明示的な観察コンテキスト

**問題**: 下位の判定関数が、既にキャプチャされた snapshot を受け取るのではなく、
`GitRepository::discover`/`.snapshot()` を自ら呼んだり Contract/Summary を
自ら再読込したりすることが多く、同一リクエストに仕える2つの関数がわずかに
異なる瞬間のリポジトリを観察してしまう可能性がある。**目標境界**: エントリ
ポイントが実際の観察フェーズ（編集前、実行後、永続化前）ごとに1つの snapshot を
キャプチャして下流に通し、下位関数は再観察せず snapshot/context パラメータを
受け取る。**互換性リスク**: 現在わずかに古い内部再読込を許容している関数は、
呼び出し元がその暗黙の再チェック（例: 操作途中の並行変更検出）に依存していないか
確認が必要。実際の編集境界をまたいで snapshot を過剰共有すれば、単なる
リファクタリングではなく正しさの退行になる。**検証**: いかなる観察フェーズも
スキップされないことを呼び出し回数のアサーションで証明し、キャプチャと使用の間の
並行変更に対する明示的なテストを行う。

### P2-A — 生命周期/ストレージ/実行/投影の責任を抽出

**問題**: `preflight_work_item_internal`、`finish_work_item_internal`、
`archive_work_item_internal`、`close_work_item_with_structured_decision_internal`
はいずれも、観察・判定・永続化を1関数本体に埋め込んでいる。**目標境界**:
本専項が提案する分割（Observation / Governance / Lifecycle / Evidence /
Execution / Projection）に従い、一度に1つの完全なユースケースを移行する ──
最小のもの（例: checkpoint）から始めて境界を証明してから finish/archive/close に
手を付ける。**互換性リスク**: 公開API シグネチャ、`.ai/` のファイルレイアウト、
履歴レコードの可読性は変更してはならない。抽出する Port は関数ごとに作るのでは
なく、実際の代替/障害注入の必要性によって正当化されなければならない。
**検証**: 既存の統合テストは変更なく成功しなければならない ── これらは既に
ファイル内容とレイアウトをアサートしており、この種の内部並べ替えに対する
適切な回帰網である。

### P2-B — 状態型と正当な遷移の厳密化

**問題**: 生命周期状態、証拠の新鮮度/適用可能性、ガバナンス判定、人間の認可、
履歴/置換済み状態は現在、少数の enum と強制された正当遷移としてではなく、
多数の関数を通じて場当たり的な文字列とブール値として表現されている。
**目標境界**: 既存の enum（`OutcomeState`、`DecisionState` は既に存在し
使われている）を再利用し、実際に欠落/無効/期限切れ/該当なしの区別が1つの
文字列に潰されている箇所にのみ狭い新しい型を追加する。**互換性リスク**:
外部プロトコル/JSON表現は明示的なバージョン/移行経路なしに変更してはならない。
履歴 evidence は新しい内部型に合わせるために書き換えてはならない。**検証**:
正当な組み合わせと不正な遷移をテーブルテストし、未知/レガシーのスキーマ
バージョンを含む履歴アーカイブレコードを新しい型に通して、読み取り専用の
履歴として引き続きパースできることを確認する。

### P2-C — マルチファイル一貫性、並行性、復旧

**問題提起のみ（本専項の指示に従い、欠陥の存在を先に仮定せず、まだ調査していない）**:
finish/archive/close とその復旧経路は操作ごとに複数ファイルを書き込む。本
Work Item は順序、冪等性キー、中断復旧を深くは監査していない。**目標境界**:
各マルチファイル操作について「コミット済み」を表す1つのレコードを特定し、
復旧がその権威あるレコードからのみ投影を再構築することを確認する。**互換性
リスク**: ここでの修正は本質的に影響範囲が大きい（コミットの意味論に触れる）
ため、無関係なリファクタリングと束ねてはならない。**検証**: 変更を提案する前に、
故障注入（各書き込みステップ後の中断、書き込み失敗のシミュレーション、同一操作の
2回実行、同一 Work Item への2プロセスの同時進行）が必要 ── 欠陥を仮定する前に
調査するという本専項自身の指示に従う。

### P3 — 物理実行とガバナンス束縛の分離

**問題**: 上述の通り、reuse 適格性の判定と物理実行/合流の仕組みは、
ガバナンス判定からの crateレベルの分離は既に存在するものの、単一の文書化された
境界所有者なしに異なる crate にまたがっている。**目標境界**: 共有/再利用された
物理実行結果それ自体が Work Item の合格状態を付与できないことを確認する
（まだ未実施）── キャッシュヒット、実行成功、ガバナンス許可は、引き続き
別々に検証される3つの事実でなければならない。**互換性リスク**:
`PhysicalSingleFlightCoordinator` の接続方法へのいかなる変更も、並行
verification の正しさとリポジトリ隔離に直接影響する。性能最適化専項自身の
発見の通り、この coordinator には現在本番の呼び出し元が存在しないため、
これを接続することは削除された挙動の復元ではなく、新しいアーキテクチャ上の
コミットメントである。**検証**: 実行共有の拡大を検討する前に、並行検証
シナリオ（同一identity、異なるidentity、失敗の伝播、キャンセル）を、
リソースピークと実行回数のアサーションとともに検証する。

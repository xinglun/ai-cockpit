---
author: AI Cockpit maintainers
title: "協作不変量カバレッジ"
description: "十の協作言語意味論的不変量を既存の自動テストカバレッジへ対応付け、正確なテストファイルと関数を引用し、残る欠落を正確に示す。"
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-681-p1-invariant-coverage-mapping
---

# 協作不変量カバレッジ

本ページは、WI-679 の協作言語契約(`docs/reference/collaboration-language-contract.ja.md`、
本 Work Item の時点ではまだデフォルトブランチに存在しない。PR #675 を参照)で述べた
十の意味論的不変量それぞれについて、今日すでに自動テストがこれを強制して
いるか、しているならどのテストかに答える。テスト基盤の重複構築を防ぎ
(まず再利用)、まだ自動的な突合が存在しない不変量を正直に示すことが目的
である。

以下の引用はすべて、本ページ執筆時点で現在のテストソースを直接読んで得た
ものであり、テスト名だけからの推測は一件もない。

## カバレッジ表

| # | 不変量 | 今日自動化されているか | テスト | 断言内容 |
| --- | --- | --- | --- | --- |
| 1 | 検証の合格 ≠ 全面的な受け入れ/合併の授権 | **はい** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | 同一の Work Item を `finish`(緑)→`archive`(黄、終結未完了)→`Deleted` 終結の記録 →`close`(明示的な `--human-decision` が必要)と遷移させ、各段階で引き渡しテキストと安定した JSON がこれらを単一の「完了」状態に潰さないことを断言する。 |
| 2 | 空の記録 ≠ リスクなし | **はい** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | リスク/テスト弱体化の節が空の Outcome をレンダリングし、人間向けテキストがデータの欠如から「リスクは見つからなかった」等の肯定的な結論を述べないことを断言する。 |
| 3 | 未知の事実は表現層で補完できない | **はい(部分的)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | `tests/adversarial/manifest.json` の意味論的ケース(現在15件)を `evaluate()` に通し、評価結果が言い回しではなくケースのデータに束縛されていることを断言する。このマニフェストに収録済みの敵対的言い回しのみを対象とし、網羅的ではない。 |
| 4 | 歴史的記録は根拠なく現在の合格/失敗になってはならない | **はい** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`、`::archived_report_tamper_is_red_and_not_reprojected_as_verified` | `runtime_historical` 記録がその歴史的な文言を保持すること(現在の失敗向けの「Repair the missing evidence」という文言が出ないこと)、および改ざんされたアーカイブ済みレポートが赤としてレンダリングされ、静かに検証済みとして再投影されないことを断言する。 |
| 5 | 同一の事実が CLI/MCP/要約/完全なレポートで一致する | **部分的——実在する欠落** | `crates/cockpit-mcp/tests/rpc.rs::mcp_work_item_outcome_returns_explicit_human_handoff_with_cli_parity`、`::mcp_blocked_outcome_exposes_the_same_recovery_facts_as_cli` | これらは `cockpit_mcp::handle_request_for_repo()` をプロセス内で直接呼び出し(ライブラリ呼び出し)、その出力を固定の期待文字列と比較する。これは MCP handler *自身*の出力の内部整合性、および文書化された CLI の文言との一致を証明するが、同一テスト内で実際に `ai-cockpit` CLI バイナリを起動して両者のライブ出力を差分比較することは一度もしていない。**今日、真にプロセスをまたいだ CLI サブプロセスと MCP handler の一致性テストは存在しない。** |
| 6 | 言語を変更しても事実/授権範囲/結果は変わらない | **はい** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`;`crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | CLI テストは `AI_COCKPIT_LANGUAGE=en/zh-CN/ja` を順に設定して実バイナリを起動し、機械可読な stdout JSON フィールドが言語間でバイト単位で同一であり、人間向けの stderr テキストのみが局所化されることを断言する。敵対的コーパステストはさらに、各意味論的ケースについて各言語5種の言い回しバリアントが同一に評価されることを検査する。 |
| 7 | 表示される次の一歩が現在の Runtime 状態/方針と一致する | **間接的** | 第1項と同じライフサイクルテスト、加えて `crates/cockpit-repository/tests/status_projection.rs::status_projection_distinguishes_archived_from_valid_closed_decision` | これらは実際の各状態遷移時に次の action に関連するフィールド(ライフサイクル段階、阻害要因)を断言するが、広範な状態行列に対してレンダリングされた「次の action」文を独立に計算した期待値と個別に突合するテストは存在しない。 |
| 8 | 既存授権が適用されるかはルールと記録で決まり、セッション切替で変わらない | **自動テスト未発見** | — | 異なる呼び出し元/セッションが、身分束縛ダイジェストに基づいて Receipt/決定を再利用する(または再利用を拒否すべき)ことをシミュレートするテストは存在しない。これは実在する欠落であり、記載済み/観測済みの挙動は `docs/reference/collaboration-scenario-matrix.json` の SCN-010/SCN-011/SCN-022 を参照。 |
| 9 | 人による決定を要する問いはすべて対象/影響/復旧条件を明示する | **専用テスト未発見** | — | `humanDecisionRequest` の形状は本番コード(`preflight`)が生成し、`docs/reference/agent-workflow.md` に記載されているが、四つの必須フィールド(what/why/options/question/resumeCondition)が常にすべて非空であることを断言するテストは見つからなかった。 |
| 10 | 要約は詳細を省略できるが阻断/重要な未知項/必要な決定を隠せない | **はい(部分的)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields`(同ファイル内の隣接する断言) | 「空は肯定的結論ではない」という半分は直接カバーしている。*記入済みの*阻断/未知項/決定が要約の圧縮で失われないことを専門に断言するテストは、第1項のライフサイクルテストで副次的にカバーされている以外には見つからなかった。 |

## 本表の読み方

- **はい**は、本リポジトリ自身のテストスイートに、不変量の記述どおりに強制する、合格しているテストが既に存在することを意味し、本ページは正確な関数を引用している。
- **部分的**は、実際のカバレッジは存在するが、不変量が述べるすべての場合には届いていないことを意味する(該当行の注記を参照)。
- **自動テスト未発見**は文字どおりの意味である:`tests/` および各 crate の `tests/` ディレクトリを検索しても見つからなかった。これは Runtime が必ずその不変量に違反しているという意味ではない——いくつかは上記テストが間接的に検査している本番コードによって構造的にも保護されている——単に、回帰が起きても今のところ捕捉するテストがないという意味である。

## 既知の欠落と推奨される後続 Work Item

1. **不変量5(入口をまたいだ一致性)**:同一テスト内で実際の `ai-cockpit` CLI バイナリをサブプロセスとして起動し(`outcome_handoff.rs` で既に使われているパターンを再利用)、同一のリポジトリ fixture に対して `cockpit_mcp::handle_request_for_repo()` を呼び出し(`rpc.rs` で既に使われているパターンを再利用)、両者がすべての安定フィールドで一致することを断言する新しい統合テストを追加する。これは既に実証済みの二つのパターンを組み合わせるだけであり、新しいテスト基盤を導入するものではない。
2. **不変量8(セッションをまたいだ授権の再利用)**:決定の受領票を一件記録した後、第二の「セッション」(同一リポジトリに対する全く新しいプロセス呼び出し)による再利用の試みを、(a)変化なし——再利用されるべき、(b)Contract/スナップショットのダイジェストが変化——新しい決定が要求されるべき、の両方についてシミュレートするテストを追加する。`crates/cockpit-repository/tests/` には既に `attach()` + `start_work_item_with_options()` という fixture パターンがあり、これを土台にできる。
3. **不変量9(`humanDecisionRequest` の完全性)**:`crates/cockpit-repository/tests/` の既存の preflight テストの隣に、`needs_human_confirmation` となるすべての preflight 結果について、その `humanDecisionRequest` の `whatHappened`/`whyItMatters`/`options`/`question`/`resumeCondition` が常にすべて非空であることを断言する、的を絞ったテストを一件追加する。
4. **不変量7(次の一歩の正しさ)**:「正しさ」が `collaboration-scenario-matrix.json` の完全な状態行列に依存するため、これは最も一般化してテストしにくい項目である。現実的な次の一歩は、汎用的な判定器を構築しようとするのではなく、その行列の `observed` な場景の部分集合に対して次の action フィールドを断言することである。

これら四点はいずれも本 Work Item では実装されていない。それぞれが境界の明確な、独立して納品可能な後続作業であり、新しいテストパターンを追加するのではなく既存のパターンを再利用するものであって、本専項自身が掲げる「既存のテスト基盤を再利用する」という方針と一致する。

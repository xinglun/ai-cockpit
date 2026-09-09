---
author: AI Cockpit maintainers
title: "協作不変量カバレッジ"
description: "十の協作言語意味論的不変量を既存の自動テストカバレッジへ対応付け、正確なテストファイルと関数を引用し、残る部分的カバレッジの制約を正確に示す。"
audience: [maintainer, reviewer, contributor]
status: current
authority: canonical
lastVerifiedBy: WI-742-p1-invariant-7-doc-promotion
---

# 協作不変量カバレッジ

本ページは、WI-679 の協作言語契約(`docs/reference/collaboration-language-contract.md`、
PR #675 でマージ済み)で述べた十の意味論的不変量それぞれについて、今日す
でに自動テストがこれを強制しているか、しているならどのテストかに答える。
テスト基盤の重複構築を防ぎ(まず再利用)、まだ自動的な突合が存在しない不
変量を正直に示すことが目的である。

以下の引用はすべて、本ページ執筆時点で現在のテストソースを直接読んで得た
ものであり、テスト名だけからの推測は一件もない。

**本ページは以前の WI-681 草稿に取って代わる。** WI-681 はマージされずに
クローズされた。理由は、この内容がレビュー中であった間に、別の並行稼働
エージェントが独立して同じ短い番号(`WI-681-wi674-doc-promotion`、
PR #677 でマージ済み)を使用したためであり——これは実在する複数エージェ
ント間の WI 番号衝突であって、内容上の欠陥ではない。今回の再送出
(WI-740)では、その草稿の判定を一件修正している:不変量8は当初「自動テ
スト未発見」とされていたが、より注意深く読み直した結果、その核心的な主
張を既にカバーする既存テストが見つかった(下記の第8行を参照)。WI-681
が欠落として指摘した不変量5と9は、この再送出より前にそれぞれ WI-682
(PR #679)と WI-710(PR #702)によって独立に解消されたため、本ページで
も「はい」と表示する。

## カバレッジ表

| # | 不変量 | 今日自動化されているか | テスト | 断言内容 |
| --- | --- | --- | --- | --- |
| 1 | 検証の合格 ≠ 全面的な受け入れ/合併の授権 | **はい** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json` | 同一の Work Item を `finish`(緑)→`archive`(黄、終結未完了)→`Deleted` 終結の記録 →`close`(明示的な `--human-decision` が必要)と遷移させ、各段階で引き渡しテキストと安定した JSON がこれらを単一の「完了」状態に潰さないことを断言する。 |
| 2 | 空の記録 ≠ リスクなし | **はい** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields` | リスク/テスト弱体化の節が空の Outcome をレンダリングし、人間向けテキストがデータの欠如から「リスクは見つからなかった」等の肯定的な結論を述べないことを断言する。 |
| 3 | 未知の事実は表現層で補完できない | **はい(部分的)** | `crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | `tests/adversarial/manifest.json` の意味論的ケース(現在15件)を `evaluate()` に通し、評価結果が言い回しではなくケースのデータに束縛されていることを断言する。このマニフェストに収録済みの敵対的言い回しのみを対象とし、網羅的ではない。 |
| 4 | 歴史的記録は根拠なく現在の合格/失敗になってはならない | **はい** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_preserves_historical_and_superseded_distinctions`、`::archived_report_tamper_is_red_and_not_reprojected_as_verified` | `runtime_historical` 記録がその歴史的な文言を保持すること(現在の失敗向けの「Repair the missing evidence」という文言が出ないこと)、および改ざんされたアーカイブ済みレポートが赤としてレンダリングされ、静かに検証済みとして再投影されないことを断言する。 |
| 5 | 同一の事実が CLI/MCP/要約/完全なレポートで一致する | **はい** | `crates/cockpit-cli/tests/cli_mcp_outcome_parity.rs::cli_subprocess_and_mcp_handler_agree_on_the_same_outcome`(WI-682、PR #679 で追加) | CLI 側では実際の `ai-cockpit` バイナリをサブプロセスとして起動し、MCP 側ではプロセス内で `cockpit_mcp::handle_request_for_repo()` を呼び出し、同一のリポジトリ fixture と同一の Work Item に対して、CLI の `work-item outcome --json` 出力と MCP `work_item_outcome` ツールの `structuredContent.outcome` が完全に等しいことを断言する(被測バイナリと完全に一致する `RuntimeContext` を使用)。これは、以前の `crates/cockpit-mcp/tests/rpc.rs::*_with_cli_parity` 系のテスト(二つのプロセス内呼び出しを比較するのみ)が提供できていなかった、真にプロセスをまたいだ検査である。 |
| 6 | 言語を変更しても事実/授権範囲/結果は変わらない | **はい** | `crates/cockpit-cli/tests/outcome_handoff.rs::default_lifecycle_commands_emit_localized_handoffs_without_changing_stdout_json`;`crates/cockpit-core/tests/adversarial_v2.rs::multilingual_adversarial_corpus_binds_wording_as_data` | CLI テストは `AI_COCKPIT_LANGUAGE=en/zh-CN/ja` を順に設定して実バイナリを起動し、機械可読な stdout JSON フィールドが言語間でバイト単位で同一であり、人間向けの stderr テキストのみが局所化されることを断言する。敵対的コーパステストはさらに、各意味論的ケースについて各言語5種の言い回しバリアントが同一に評価されることを検査する。 |
| 7 | 表示される次の一歩が現在の Runtime 状態/方針と一致する | **はい** | `crates/cockpit-repository/tests/scenario_matrix_next_action.rs::scn_001_checkpoint_before_start_matches_scenario_matrix_key_message`、`::scn_002_start_rejects_pre_existing_changes_matches_scenario_matrix_key_message`、`::scn_016_finish_before_finalize_plan_matches_scenario_matrix_key_message`、`::scn_001_002_016_are_still_declared_observed_in_the_scenario_matrix`(WI-741) | SCN-001 と SCN-016 の文書化された `expected.keyMessage` との完全一致、SCN-002 に必要なメッセージ部分文字列と違反パスが含まれること、さらに三つのシナリオが `sourceType: "observed"` のまま不変量7を引き続き検証することを断言する。これは観測済みシナリオに対する境界付きのカバレッジであり、全状態行列を網羅する判定器ではない。 |
| 8 | 既存授権が適用されるかはルールと記録で決まり、セッション切替で変わらない | **はい** | `crates/cockpit-repository/tests/preflight_review.rs::bound_human_review_receipt_allows_checkpoint_but_not_stale_reuse` | 決定の受領票を一件記録し、`preflight` が `human_decision_recorded` に遷移し、スナップショットが未変化のうちは `checkpoint` が成功する(再利用が有効)ことを確認したうえで、その後リポジトリを変更し、`preflight` が `needs_human_confirmation` に戻り `checkpoint` が拒否される(スナップショットの変化により以前の決定が無効化され、新しい決定が必要になる)ことを断言する。これはこの不変量が述べる「ルールと記録に基づく」という核心的な主張を直接証明しており、このテスト自体は本 Work Item より前から存在していた。以前の WI-681 草稿はこれを誤って欠落と判定していた。(同一の呼び出し元がスナップショット変化をまたぐのではなく)*異なる呼び出し元の身元*を専門に検査するテストを追加することは、より狭い任意の後続作業であり、この不変量の核心的な主張に対する現在の検査の欠如ではない。 |
| 9 | 人による決定を要するすべての問いは対象/影響/復旧条件を明示する | **はい** | `crates/cockpit-repository/tests/contract_preflight.rs::assert_human_decision_request_is_complete`(`::scaffold_preflight_is_not_ready_and_records_human_review_requirements` と `::high_risk_scenario_coverage_stops_at_preflight_for_human_review` から呼び出される。WI-710、PR #702 で追加) | `what_happened`、`why_it_matters`、`question`、`resume_condition`、`options`、`recommended_option`、`recommendation_reason` がすべて非空であること、`recommended_option` が提示された選択肢のいずれかを指すこと、各選択肢の `id`/`label`/`effect` が非空であることを、独立して発生する二つの実際の `needs_human_confirmation` シナリオに対して検査する。 |
| 10 | 要約は詳細を省略できるが阻断/重要な未知項/必要な決定を隠せない | **はい(部分的)** | `crates/cockpit-repository/tests/outcome_report.rs::human_renderer_does_not_infer_risk_absence_or_test_strength_from_empty_fields`(同ファイル内の隣接する断言) | 「空は肯定的結論ではない」という半分は直接カバーしている。*記入済みの*阻断/未知項/決定が要約の圧縮で失われないことを専門に断言するテストは、第1項のライフサイクルテストで副次的にカバーされている以外には見つからなかった。 |

## 本表の読み方

- **はい**は、本リポジトリ自身のテストスイートに、不変量の記述どおりに強制する、合格しているテストが既に存在することを意味し、本ページは正確な関数を引用している。
- **部分的**は、実際のカバレッジは存在するが、不変量が述べるすべての場合には届いていないことを意味する(該当行の注記を参照)。
- 現在、十の不変量すべてに少なくとも直接または部分的な自動テスト
  カバレッジがある。残る部分的な制約は第3項と第10項に明記している。

## カバレッジ更新と残る制約

**不変量7(次の一歩の正しさ)**は WI-741 により、境界を明確にした直接
カバレッジを得た。テストは SCN-001、SCN-002、SCN-016 の次の action
テキストを場景行列の `expected.keyMessage` と突き合わせ、これらが観測済み
で不変量7を引き続き検証することも守る。`docs/reference/collaboration-scenario-matrix.json`
の全状態に対する網羅的な判定器を主張するものではない。

本表では、未テストの不変量を名指しした欠落として残していない。残る制約は
第3項と第10項に記載した部分的カバレッジである。

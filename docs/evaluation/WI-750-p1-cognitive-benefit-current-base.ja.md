# WI-750：P1-A 認知的な効果の評価

状態：評価資料を提供済み。認知的な効果は未検証です。

この Work Item は **Calibrated Human-Agent Trust** のための再現可能な比較を
準備します。実際の参加者はなく、読む時間の短縮、誤った進行の減少、リスク検出の
改善は主張しません。外部 adopter や別 repository のデータも読み書きしていません。

## 固定した方法と回答キー

タスク集合と回答キーは比較前に
[`tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py`](../../tests/evaluation/WI-750-p1-cognitive-benefit-current-base.py)
へ固定しました。実際の archive `OutcomeV2` と `TaskOutcomeReport` を読み、現在の
source-built CLI で `--view summary` と `--view full` を生成し、次を比較します。

- 検証、ライフサイクル、人間の判断、ガバナンスシグナル；
- evidence 参照と四つの summary セクション；
- 人間の次のアクションと重要な不確実性の可視性；
- 英語、中国語、日本語の見出し。

完全レポートは監査用 projection、summary は既定の読者用 projection です。自動の
full 呼び出しは整合性確認であり、参加者の閲覧イベントではありません。元の evidence
本文はデータとしてのみ扱い、指示や権限の根拠にはしません。

## 固定タスク集合

| タスク | 実際の Outcome | 境界の事実 | 回答キー |
| --- | --- | --- | --- |
| 正常完了 | `WI-663-wi659-outcome-trust-replacement` | `finish_ready/green` | 緑の検証は merge や release の承認ではなく、evidence とリスク境界を確認する。 |
| 検証済み、判断待ち | `WI-658-wi656-outcome-trust-repair` | `finish_ready/green`、close 判断なし | 人間の判断は未記録。evidence を確認し明示的な close 判断を記録する。 |
| scope 超過 | `WI-714-wi713-current-base-revalidation` | `tests/conformance/fixtures/scope-exceeded/input.json` | 停止する。scope は境界であり、続行の権限ではない。 |
| evidence の期限切れまたは identity 不一致 | `WI-423-ci-convergence` | `tests/conformance/fixtures/contradictory-evidence/input.json` | 停止し、identity が一致する新しい evidence を取得する。 |
| test weakening シグナル | `WI-662-p0-benchmark-evidence` | `tests/conformance/fixtures/test-weakening/input.json` | 検査範囲と scan 結果を確認する。空のリスク記録は「弱化なし」ではない。 |
| 未検証範囲 | `WI-139A-preflight-review` | Contract に未検証シナリオと計画がある | 未検証範囲を残し、完了を主張する前に計画を実行する。 |
| 履歴上の close 済みタスク | `WI-743-wi715-p1-current-base-redelivery` | close 判断と履歴 evidence | 履歴を保持し、current result が必要な時だけ再検証する。保証レベルを上げない。 |

archive の元状態と現在 Runtime の projection は生成 evidence に両方保存します。履歴
または foreign Runtime の projection が履歴、未知、未準備として表示されるのは信頼境界で
あり、現在の緑の成功として扱いません。

## 結果と制限

結果は `.ai/evidence/WI-750-p1-cognitive-benefit-current-base.json`、読者向けの要約は
`.ai/evidence/external/WI-750-p1-cognitive-benefit-current-base.md` に保存します。

今回の実行は固定タスク 7 件、summary/full の自動整合性違反 0 件、重要情報の可視性違反
0 件、参加者 0 名、検証済みの認知的効果なしでした。状態と次のアクションの正答時間、
誤った release、重要リスクの見落とし、緑を安全または承認と誤解した回数、参加者の full
閲覧回数は未測定です。full の 14 回は自動 oracle の呼び出しであり、ユーザー行動では
ありません。

再実行：

```sh
bash tests/evaluation/WI-750-p1-cognitive-benefit-current-base_test.sh
```

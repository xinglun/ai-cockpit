# Work Item 間の調整

この機能は候補 Runtime 専用です。Runtime `0.2.105` は既存ライフサイクルを
担当し、調整レコードを読みません。候補 Runtime は capability discovery、
repository-local な読み書き、impact admission、正確な composition verification
を担当します。これは双方向互換性の約束ではありません。

## 対応範囲

Git の common directory を解決するため、同じ repository の linked worktree を
調整できます。独立 clone とマシンをまたぐ調整は未対応です。レコードは
`.ai-cockpit/coordination/v1/` に置き、repository、WI、Contract、worktree/head、
候補 Runtime capability、execution generation に束縛します。

登録と inspect は canonical な Git topology と active Contract からこれらの事実を
再観測します。呼び出し側が宣言した repository id、branch、head、Contract digest、
evidence path だけでは准入になりません。後続世代で head、Contract、declaration
が変わると、重複排除された Impact event が追加されます。Outcome の公開は消費者を
無効化せず、recovery は現在の provider generation/head/Contract digest を記録するため、
履歴を削除せずに旧 event を新世代から解決できます。

通常の単一 WI は高速経路のままです。明示的に参加しない WI では調整 store を
scan せず、書き込みません。

読み取り専用の確認：

```text
ai-cockpit work-item coordination inspect --repo <path>
```

登録、impact 報告、安全な pause、resume、recovery consumption、composition は
明示的な書き込み操作です。recovery consumption は event/provider generation/
consumer generation の完全一致で idempotent になり、古い generation を拒否します。
impact は影響を受けた consumer だけを止め、無関係な WI は継続できます。pause の
request、acknowledged、safely paused、unavailable/expired、resumed は区別します。

composition は admission を再確認し、安全に pause された対象を検証プロセス起動前に
拒否します。Runtime は target topology、参加者の登録済み head/Contract、必須 check
の一意で完全な coverage を検証し、観測事実から前提条件を計算します。共有の bounded
executor で有限 timeout と bounded output を使い、起動前に in-progress attempt を
保存し、各 node、timeout、中断からの復旧情報、cleanup 結果を保存します。全 identity
が一致する node だけを再利用するため、完全な再実行はプロセスを起動せず、局所変更は
影響を受けた node だけを再実行します。

CLI と MCP は同じ repository service を利用します。Outcome では実装、composition、
target merge、cleanup を分離し、比較のない効果は主張しません。

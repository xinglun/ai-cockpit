# WI 間調整機能

この機能は、依存宣言、impact 報告、安全境界での調整、正確な composition 検証、
人間向け Outcome 投影を一つのループにします。WI ごとの opt-in であり、開始前の
「確認要求」を新しい governance gate にはしません。

repository service が domain boundary、CLI と MCP が adapter、Outcome が projection
です。query は read-only、登録・impact・状態遷移・recovery consumption・composition
は明示的な write です。

対応する topology は同一 Git common directory の linked worktree です。固定 Runtime
は lifecycle compatibility を担当し、候補 Runtime が collaboration capability を
advertise した場合だけレコードを読み書きできます。

登録は Git identity、active Contract、head、branch、regular evidence を Runtime が
再観測してから有効になります。Composition は Git common directory に共有記録を残し、
bounded executor を使い、実際の node 単位の再利用と cleanup を Outcome に投影します。

# WI 間調整機能

この機能は、依存宣言、impact 報告、安全境界での調整、正確な composition 検証、
人間向け Outcome 投影を一つのループにします。WI ごとの opt-in であり、開始前の
「確認要求」を新しい governance gate にはしません。

repository service が domain boundary、CLI と MCP が adapter、Outcome が projection
です。query は read-only、登録・impact・Outcome 公開・状態遷移・recovery consumption・
composition は明示的な write です。Outcome 公開は現在の登録 generation と正確な
evidence bytes に束縛されます。

対応する topology は同一 Git common directory の linked worktree です。インストール済み
Runtime `0.2.113` は lifecycle compatibility を担当し、coordination store を読みません。
候補 Runtime が collaboration capability を advertise した場合だけレコードを書き込み、
消費できます。古い binary が候補専用 Contract check-coverage field を読み取り、または
強制すると仮定してはいけません。

登録は Git identity、active Contract、head、branch、regular evidence を Runtime が
再観測してから有効になります。Composition は Git common directory に共有記録を残し、
bounded executor を使います。観測済みの executable、command、有効 environment、宣言
入力 bytes、dependency receipt が一致する場合だけ node を再利用し、node 入力が不明
なら reuse を無効にします。Outcome は applicability、実際の merge、cleanup、reuse を
分けて投影します。

composition が中断された場合、実行中 verifier の process group を永続記録します。
group が生存中、または状態を確認できない間は retry で一時 worktree を保持します。Unix
では同じユーザーの process cwd と worktree 内の open file handle も調べ、detached session
の子プロセスが tree を参照している間は cleanup を阻止します。検査が不完全なら fail closed
です。Windows では bounded executor の kill-on-close process job が子孫を管理します。
Outcome は整合した terminal attempt がある場合だけ passed と reusable checks を示します。

composition coverage は digest に束縛された必須 `verification` check が宣言する
`coversScenarios` または `coversConstraints` から導出します。呼び出し側の label は coverage
を付与しません。execution repository は coordination store と同じ Git common directory に
属する必要があり、repository id をコピーした独立 clone は拒否されます。

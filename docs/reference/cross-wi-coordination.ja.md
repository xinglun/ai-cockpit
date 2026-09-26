# Work Item 間の調整

この機能は候補 Runtime 専用です。インストール済み Runtime `0.2.113` は lifecycle
owner のままで、調整レコードや候補専用の check-coverage metadata を読みません。
候補 Runtime は capability discovery、repository-local な読み書き、impact admission、
正確な composition verification を担当します。これは双方向互換性の約束ではなく、
古いインストール済み binary が候補用フィールドを parse または強制すると仮定してはいけません。

## 対応範囲

Git の common directory を解決するため、同じ repository の linked worktree を
調整できます。独立 clone とマシンをまたぐ調整は未対応です。レコードは
`.ai-cockpit/coordination/v1/` に置き、repository、WI、Contract、worktree/head、
候補 Runtime capability、execution generation に束縛します。composition 実行時には
execution repository と coordination store が同じ Git common directory を共有することも
検証します。repository id をコピーした独立 clone だけでは不十分です。

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

調整状態を変更する操作は明示的な write です：

```text
ai-cockpit work-item coordination register --repo <path> --input registration.json
ai-cockpit work-item coordination report-impact --repo <path> --input event.json
ai-cockpit work-item coordination publish-outcome --repo <path> --id <wi> --generation <n> --outcome-id <outcome>
ai-cockpit work-item coordination request-pause --repo <path> --input request.json
ai-cockpit work-item coordination acknowledge --repo <path> --request-id <id> --state acknowledged
ai-cockpit work-item coordination resume --repo <path> --id <wi> --generation <n>
ai-cockpit work-item coordination recover --repo <path> --event-id <id> --consumer-work-item-id <wi> --consumer-generation <n>
ai-cockpit work-item composition --repo <path> --id <wi> --generation <n> --input composition.json
```

登録、impact 報告、安全な pause、resume、recovery consumption、composition は
明示的な書き込み操作です。recovery consumption は event/provider generation/
consumer generation の完全一致で idempotent になり、古い generation を拒否します。
impact は影響を受けた consumer だけを止め、無関係な WI は継続できます。pause の
request、acknowledged、safely paused、unavailable/expired、resumed は区別します。
各 state transition は、まず呼び出し元の request ID を検証してから request record を読み込みます。
読み込み後は保存済み target WI ID を安全な path component として検証してから registration path を
解決します。registration を読み込んだ後、埋め込み workItemId が安全な target と一致することを
先に確認し、その後に registration facts の検証や Contract path の解決を行います。改ざんまたは
identity が一致しない record は拒否され、request bytes は変更されません。

request-scoped observation が保証するのはその request 内の一貫性だけであり、process 間で
coordination event を配送しません。新しい process が読むのは、明示的な write によって
永続化された event だけです。

Outcome の公開は、現在の登録 generation と宣言済み outcome に対する明示的な write
です。Runtime は現在の consumer が要求する verification receipt を検証し、正確な
evidence bytes に束縛した `OutcomePublished` event を追加します。read-only query は
公開、修復、event 消費を行いません。公開後に evidence bytes が変わると束縛が崩れ、
verification dependency を満たせません。

### MCP identity fields

`work_item_coordination` schema は action ごとに許可する field を制約します。
`publish-outcome` は `providerWorkItemId`、`providerGeneration`、`outcomeId` を使います。
旧 `workItemId`/`generation` の組はこの action だけで legacy alias として受け付けます。
`resume` では同じ名前が再開対象の Work Item を指します。`recover` は `eventId` と
`consumerWorkItemId`、`consumerGeneration` を使い、provider identity は不変 event から
解決します。余分な field や identity の混在は拒否されます。

```json
{"action":"publish-outcome","providerWorkItemId":"WI-PROVIDER","providerGeneration":3,"outcomeId":"api"}
{"action":"recover","eventId":"impact-1","consumerWorkItemId":"WI-CONSUMER","consumerGeneration":2}
```

composition は admission を再確認し、安全に pause された対象を検証プロセス起動前に
拒否します。一時 linked worktree の準備後も各 action boundary で admission と参加者/target
identity を再検証します。再利用 node は共有調整 lock 内で admission と identity を再確認し、
再利用記録の追加と永続化まで lock を保持します。実行 node も同じ lock を child process
生成まで保持します。pause または impact が先に確定した場合は再利用/起動を拒否し、いずれかの
action 受理後に到着した request は次の安全境界で反映します。
Runtime は target topology、参加者の登録済み head/Contract、必須 check
の一意で完全な coverage を検証し、観測事実から前提条件を計算します。active Contract の
必須 `verification` check は `coversScenarios` と `coversConstraints` を宣言できます。
Contract digest と正確な必須 check identity に束縛された値だけを coverage として扱います。
composition input の label は説明用 assertion にすぎず、coverage の証拠ではありません。
未対応 label または Contract 側の mapping 欠落は process 起動前に fail closed します。
必須 Contract check は空でない `CompositionCommand.environment` overlay も拒否します。
現在の typed `VerificationCheck` には environment 宣言がないため、caller の値をその check に
束縛できません。identity と reuse のために実効環境を観測しても、overlay で Contract 必須 check を
変更する権限にはなりません。
共有の bounded
executor で有限 timeout と bounded output を使い、起動前に in-progress attempt を
保存し、各 node、timeout、中断からの復旧情報、cleanup 結果を保存します。全 identity
が一致する node だけを再利用します。呼び出し側の identity digest は reuse を許可せず、
実行中 node と verifier process group の identity も永続化します。process group の停止を
確認できるまで retry は一時 worktree を保持します。Unix では同じユーザーの process cwd と
worktree 内の open file handle も確認し、tree を参照する detached session の子孫を検出します。
検査が不完全なら fail closed です。Windows では executor の kill-on-close process job が
子孫を管理します。Outcome は整合した terminal receipt に限って passed または reusable checks を示します。
Runtime は target tree、lock/configuration files、解決された executable、有効な環境、
宣言された入力ファイルを観測します。観測済み executable、command、有効 environment、
宣言入力 bytes、upstream receipt が成功済み predecessor と一致する場合だけ再利用し、
node 入力が欠落または観測不能なら reuse を無効にします。同一入力ならプロセスを起動せず、
宣言入力の変更時は該当 node と依存 node を再実行します。

CLI と MCP は同じ repository service を利用します。Outcome では実装、仮 composition、
現在の composition applicability、実際の target merge、cleanup を分離します。target
または participant の事実が変わった場合も履歴上の成功は保持しつつ stale と示します。
比較のない効果は主張しません。

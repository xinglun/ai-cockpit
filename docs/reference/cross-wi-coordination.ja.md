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

composition は admission を再確認してから、一時 linked worktree に宣言順で構築
します。前提条件が満たされない場合は高コスト検証を起動しません。source、依存、
interface、configuration、toolchain、lockfile、generated input、environment、
verifier、command identity と前回の明確な成功が一致した場合だけ再利用します。

CLI と MCP は同じ repository service を利用します。Outcome では実装、composition、
target merge、cleanup を分離し、比較のない効果は主張しません。

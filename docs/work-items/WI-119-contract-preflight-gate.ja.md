# WI-119 — Contract preflight human-review gate

## 目的

Rust Runtime の事前 Contract boundary を reference Agent workflow に合わせ、不確実な場合は
実装を停止して human review を要求し、ready として黙って扱わない。

## 範囲

- 互換性のある Contract `sources` と `verification` 宣言を追加する。
- 不完全な Contract は `reviewState: needs_human_confirmation` 付き yellow とし、bind された preflight receipt を保存する。
- checkpoint は green または `verification_pending` yellow だけを許可し、human-review yellow と red は fail closed にする。
- repository/Work Item/Contract/snapshot binding を維持し、CLI/MCP と三言語文書を同期する。

## 範囲外

Release publication、global Agent/MCP configuration、過去の archive Work Item bytes の書き換え。

## 受入れ

1. `work-item new` 後の `preflight` は ready にならず human fields を示す。
2. scaffold に authority、intent、scope、acceptance がない場合 checkpoint を越えられない。
3. 宣言された verification が不足する場合は `verification_pending` のまま evidence 収集だけを許可する。
4. Contract または snapshot が変わったら preflight をやり直す。
5. CLI と MCP は同じ review state、blockers、unknowns、next action を公開する。

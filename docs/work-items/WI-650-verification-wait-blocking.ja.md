---
author: AI Cockpit maintainers
title: WI-650 — 検証子プロセスのブロッキング待機化
description: 固定10msポーリング待機をUnix上でブロッキング待機に置き換え、計測済みの前後比較を添える。
workItemId: WI-650-verification-wait-blocking
audience:
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-650-verification-wait-blocking
terminalArchive: .ai/work-items/archive/WI-650-verification-wait-blocking.contract.json
terminalVerification: .ai/evidence/WI-650-verification-wait-blocking.verification.json
terminalFinalization: .ai/decisions/WI-650-verification-wait-blocking.finalize.json
terminalDecision: .ai/decisions/WI-650-verification-wait-blocking.close.json
---

# WI-650 — 検証子プロセスのブロッキング待機化

本 Work Item は AI Cockpit 性能最適化専項の P1 にあたる。P0 の事前調査で発見した
固定間隔のビジーポーリング待機（`crates/cockpit-verification/src/lib.rs` の
`execute_captured` 内、子プロセス待機ループ）を除去する。

## 発見した欠陥と計測

すべての検証子プロセスの待機ループは次の通りだった:

```rust
let deadline = Instant::now() + Duration::from_secs(MAX_EXECUTION_SECONDS);
let (status, mut timed_out) = loop {
    match child.try_wait() {
        Ok(Some(status)) => break (Some(status), false),
        Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
        Ok(None) => { terminate_process_tree(&mut child, child_id); break (child.wait().ok(), true); }
        Err(_) => { terminate_process_tree(&mut child, child_id); break (None, false); }
    }
};
```

本番コードを変更する前に、独立したマイクロベンチマーク（CLI を介さず `/bin/true` と
`sleep 1` を直接タイトループで起動し、無関係な起動/git/identity のノイズを排除）で
このループのコストを定量化した:

| シナリオ | ビジーポーリング | ブロッキング `wait()` |
|---|---|---|
| `true` (n=200) | 平均 12.192 ms、p50 12.523 ms | 平均 1.188 ms、p50 1.193 ms |
| `sleep 1` (n=5) | 平均 1013.309 ms | 平均 1009.251 ms |

ほぼ瞬時に終わるコマンドでは、ビジーポーリングループが約11msの純粋な待機時間を
追加していた（このシナリオでは約10倍の latency 膨張）。数秒かかるコマンドでは
両者は統計的に区別できず、ポーリング間隔の相対的な重みから予想される通りだった。

## 修正内容

- `execute_captured` の待機ロジックを `wait_for_child(child, child_id,
  deadline)` として切り出し、`#[cfg(unix)]` 実装と、変更していない
  `#[cfg(windows)]` 実装に分けた。
- Unix では、子プロセスを専用スレッドに move し、そのスレッドがブロッキングの
  `child.wait()` を呼んで結果を `mpsc::sync_channel` で送る。呼び出し側は
  1回だけ、締切までの残り時間で `recv_timeout` する。タイムアウト時は pid
  （`Copy` 型で、move 済みの `Child` 値を必要としない）による `libc::kill` で
  プロセスを終了させ、子孫プロセスの終了処理は従来通り行った上で、チャンネルから
  最終的な終了ステータスを取得する。これにより、既存のタイムアウト・
  プロセスツリー終了・出力キャプチャの保証はすべて維持される（出力キャプチャは
  元々ビジーな sleep ループではなく、有界の `libc::poll` を使用していたため
  変更していない）。
- Windows では `wait_for_child` は以前のループと文字通り同一である。本
  Work Item では Windows 環境が利用できず**検証していない**ため、検証なしに
  変更することは正当化できないリスクであり、意図的に手を加えていない。
- `terminate_process_tree`（`&mut Child` を必要とし、Unix側でスレッドに
  move された `Child` とは両立しない）は `#[cfg(windows)]` 専用にした。

## 正しさの検証

- 3つの新規ユニットテスト（`crates/cockpit-verification/src/lib.rs` の
  `#[cfg(all(test, unix))] mod wait_for_child_tests`）が、実際の300秒の
  `MAX_EXECUTION_SECONDS`（end-to-endでの検証は非現実的）の代わりに短い
  合成締切を使って `wait_for_child` を直接呼び出す: 締切に十分間に合う
  コマンド（timed_out にならず、正しい終了コード）、100msの締切を超える
  コマンド（`sleep 5`。5秒ではなく2秒未満でkillされ `timed_out=true` になる
  ことを確認）、終了コードの保持（`sh -c 'exit 7'` がコード7を報告）。
  3つ合計で約0.1秒で完了する。
- 既存の `cockpit-verification` テストスイート全体（9ファイル56テスト。
  `bounded_execution_reports_plan_and_process_telemetry`、
  `detached_descendant_pipe_is_cancelled_and_fails_closed` など
  実行/タイムアウトに隣接するテストを含む）は変更なく成功する。
- `cargo test --locked --workspace` は変更なく成功する（120件の test result
  ブロック、失敗0件）。

## 計測された性能(参考情報)

2026-09-08、macOS arm64、`ai-cockpit verify --repo <fixture> --command true`
（常にfreshな custom command で、reuse されない）を、小さな attach 済み
フィクスチャリポジトリに対して12イテレーション、baseline（インストール済み
v0.2.87）対 本ビルドで計測:

- baseline: 典型的に約104〜108 ms（123msの外れ値1件を除く）
- candidate: 典型的に約94〜98 ms（初回呼び出しの505msの外れ値1件を除く）

約10msのエンドツーエンド改善は、独立したマイクロベンチマークで得られた
約11msという結果とよく一致しており、この改善が除去したポーリング待機に
起因するものであり、計測ノイズではないことを裏付けている。これはローカルな
プロセス latency の観測であり、provider や enterprise の保証ではない。

## 対象外

- Windows の待機パス（変更なし、本 Work Item では未検証）。
- 有界並列ファイル読み取り/ハッシュ（別のP1候補であり、本 Work Item では
  着手していない）。
- 出力ストリームキャプチャ、依存関係スケジューリング、リソース予算への変更。

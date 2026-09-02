---
author: AI Cockpit maintainers
title: "Troubleshooting と recovery"
description: "AI Cockpit の代表的な stop state と安全な次の action。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: WI-512-reference-docs-batch-33
capabilityClaims:
  - recovery
---

# Troubleshooting と recovery

| Observation | 意味 | 安全な次の action |
| --- | --- | --- |
| `state: unattached` | 有効な `.ai/cockpit.toml` がない。 | target を review し `attach --repo <path>`。 |
| `calibration_required` | profile は検出されたが human confirmation 前。 | command を review し `profile confirm`。 |
| Preflight `yellow` | evidence 不足または confirmation が必要。 | blockers/safe actions を読み Contract を修正するか decision を得る。 |
| Preflight `red` | scope、authority、protocol、repository state が invalid。 | 停止し、指定された fact/authority を修復する。 |
| `finish` が receipt missing/stale | current snapshot の passed Work Item verification がない。 | 最終 edit 後に `verify --work-item <id>`。bypass しない。 |
| `finish_ready` の Work Item が archive 前に stale になった | verification が bind した snapshot の後で repository が変わった。 | summary/receipt を編集しない。historical bytes を残し、現在の snapshot から新しい認可済み Work Item を作る。 |
| stale predecessor を再試行すべきでない | evidence が履歴であり、bind 済み successor が作業を引き継いでいる。 | identity-bound な `supersede` recovery decision を記録し、predecessor を履歴項目として archive/close する。古い evidence を書き換えたり再検証したりしない。 |
| `archive`/`close` が失敗 | governance が green でない、または archive identity が invalid。 | active record を残し evidence を修復して失敗した step を再実行する。 |
| Verification が reuse でなく rerun | identity binding が変化、または reuse が未承認。 | rerun を安全な動作として扱い receipt reason を確認する。 |
| MCP が repository binding を要求 | repository-bound adapter なしで server を起動した。 | `mcp --repo <path>` を設定し path を明示する。 |
| Release asset/tag がない | public distribution evidence が未準備。 | install を停止し、immutable Release と matching asset を待つ。 |

`.ai` record、receipt、`index.pending` を削除して status をきれいに見せてはいけません。missing、malformed、stale、矛盾した
evidence は意図的に fail closed になります。

## Installation と toolchain の境界

`attach`、`profile confirm`、`agent doctor` が停止した場合は、示された repository fact を確認し、明示的な `--repo` で同じ command を再実行します。Runtime は project の JDK、Gradle、Xcode、CocoaPods、Node、その他の external toolchain を install/切替しません。project command の不足は adopter configuration の問題であり、Contract を弱めたり workspace binary に置き換えたりする理由にはなりません。

repository に active Work Item がある場合は、upgrade や新しい Work Item の前に finish/archive します。linked worktree、remote の既定 branch、finalization receipt がない場合は停止して record を保全します。Recovery は identity-bound な successor/retry path を使います。reference source の Make/Python wizard command は Rust Runtime command ではありません。installed CLI と repository が宣言した verification command を使ってください。

すでに `finish_ready` の Work Item に暗黙の rewind 操作はない。rewind は
state history を曖昧にするためである。変更後の snapshot 用に successor Work
Item を作り、古い receipt を historical evidence として参照する。

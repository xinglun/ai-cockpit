---
author: AI Cockpit メンテナー
title: WI-1031 — Work Item 横断の受け入れ修正
description: 独立レビューで見つかった協調 identity、evidence、action admission、Outcome の不足を修正する。
workItemId: WI-1031-cross-wi-acceptance-corrections
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: close_pending
authority: human-authorized
lastVerifiedBy: WI-1031-cross-wi-acceptance-corrections
---

# WI-1031 — Work Item 横断の受け入れ修正

## 意図

Runtime は現在、この archived Work Item を historical verification 済み、human close pending と projection しています。選択された recovery lineage は WI-1032、WI-1033 へ続きます。Runtime が正確な lineage close を記録するまでは、archive を terminally closed と表示しません。

## 範囲

- 実際の repository、executor、Contract、dependency facts から composition reuse identity と必須 check の網羅性を計算する。
- 現在の provider identity に結び付いた有効な成功 verification evidence と、実際の target merge facts を要求する。
- impact 登録を復旧可能にし、outcome 単位の admission と三段階の依存伝播を実装する。
- 履歴 composition を保持しつつ、stale、仮 composition、target merge、安全 pause を別々に表現する。
- 通常の単一 WI による逐次実行を維持する。

## 対象外

Release、tag 変更、artifact 公開、Runtime upgrade、WI-1030 の不変履歴の変更、global Agent/MCP 設定、無関係な協調アーキテクチャの再設計。

## 受け入れ条件

1. 実際の toolchain または environment が変わった場合、古い呼び出し元 JSON は reuse を許可しない。観測できない実行入力では reuse を無効にする。
2. process 起動前に必須 scenario、participant、composition order、dependency closure を網羅し、consumer worktree から解決済み `main` を target にできる。
3. verification dependency は現在の provider identity に結び付いた対応済み成功 receipt のみを受け入れ、`MergedTarget` は実際の target 統合を示す。
4. registration/event の中断を復旧でき、同じ WI 内の action は outcome ごとに判定し、impact を三段階の依存へ伝播する。
5. Outcome は履歴を保持しながら stale を明示し、仮 composition と target merge を分け、pause admission と一致する。
6. 通常の単一 WI workflow は引き続き逐次実行され、cross-WI coordination state を要求しない。

## Evidence

現在までの重点証拠は通過している。composition/reuse（17 tests）、collaboration admission/projection（25 tests）、coordination storage/recovery（14 tests）、repository gate manifest 回帰、および 2 つの linked worktree を使った実プロセス受け入れを実行済み。実プロセス受け入れでは registration が 2 件、重複排除された impact event が 1 件となり、初回 composition は検証プロセスを 1 件、同一入力の再実行は 0 件起動した。実行可能ファイル／環境変更、選択的・推移的 invalidation、起動前の不正依存順序拒否、通常の単一 WI 逐次検証も含む。workspace 全体テスト、Clippy、完全な Runtime/Cargo/CI canonical gate、独立 PR review、merge、cleanup は未完了。Release 前で停止する。詳細は[仕様](WI-1031-cross-wi-acceptance-corrections/spec.md)と[実装計画](WI-1031-cross-wi-acceptance-corrections/implementation-plan.md)を参照。

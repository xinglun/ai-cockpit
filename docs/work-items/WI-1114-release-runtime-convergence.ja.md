---
author: AI Cockpit maintainers
title: "WI-1114-release-runtime-convergence — Runtime リリース収束"
description: "Mac 動作、CI コスト、公開ヘルプ、Sentinel 受入れ、検証から終了への信頼できる証拠連携を整える。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1114-release-runtime-convergence
status: in_progress
authority: native-contract
lastVerifiedBy: WI-1114-release-runtime-convergence
---

# WI-1114 — Runtime リリース収束

この WI は五つの subtask に限定します。範囲、権限、admission は native Contract と
Runtime の現在の projection に従います。本ページは進捗であり権限を付与しません。
WI-1113 は `main` に納品済みですが native closeout は未完了です。原証拠と復旧資源を
保持し、この WI の開始で自動的に retire しません。

## 範囲

1. **Mac:** macOS arm64 で対応する supervision 境界を実装し実測します。通常終了、timeout、
   親切断、protocol failure、detach 後も pipe を保持する子孫を検証します。制限と不確実性を
   明示し、Linux と Windows の保証を維持します。
2. **CI コスト:** compile、実行、証拠 reuse、cleanup の時間を分けて測ります。証明できる重複
   だけを除き、完全な target coverage を維持します。共有実装と等価な否定テストで妥当な場合
   のみ script を Rust 化します。言語変更そのものは高速化の証拠ではありません。
3. **Help:** Clap から公開 command 一覧を生成し、実際の短い/長い help と discovery description
   を検証します。英語、日本語、簡体字中国語 reference を揃え、private supervisor を公開しません。
4. **Sentinel:** 明示的に承認した正式公開後、独立した consumer copy で同一の immutable package、
   binary、inspect identity を固定します。#1004、#1041、関連 amendment-provenance 回帰を別々に
   判定し、元の入力と失敗を保持します。
5. **証拠と closeout:** 後続 metadata/集約エラーで失われる前に実際の実行事実を永続化します。
   CI 証拠は provider、source、Runtime、各 command の等価性を明示的に確認して接納し、本当の
   gap は実行します。進捗出力は証拠でも権限でもありません。

## 検証と納品順序

小さい実際の正例/負例、独立 review、最終 source に bind した check、必要な platform CI の
順に進めます。局所 test の成功は全体や別 platform の成功ではありません。比較可能な CI13
quality **4,701 秒**を基準に、30%以上短縮かつ55分以下を目標とし、cache 条件と維持した
coverage を報告します。この改善はまだ実証されていません。

quality CI の**総予算は main を含む8回**、release workflow は**総予算2回**です。現在は
**0/8、0/2**を使用しています。local command は1 worker、各900秒以内、有限で保護された
process cleanup、空き disk 1 GiB 以上を守ります。失敗や不確実な結果を保持し、根拠なく
retry したり復旧証拠を削除したりしません。

実装と source CI の後、正確な version/asset の承認を得て正式公開します。その同じ不変の
正式 package を Sentinel が受け入れてから個別に判定します。candidate、plan-only、AI Cockpit
の self-test は consumer acceptance を代替しません。必要な受入れが未完了なら WI は進行中です。
`v1.1.0` は提案であり、承認済み tag/release ではありません。

## 現在の状態と境界

sequence 2 の実際の owner 確認は native Runtime に記録済みで、初期 checkpoint も完了しています。
実行事実の永続化と help の局所 module に定向証拠があります。全体の中断復旧、信頼できる CI
接納、Mac 受入れ、高速化の実測、正式 package の consumer acceptance は未完了です。この WI の
quality CI と release workflow はまだ開始していません。

現在の control は正式公開、共有/production Runtime の置換、不可逆削除、早期の issue close、
架空の finish/archive/close を許可しません。正確な Runtime 切替と native human decision は
それぞれの binding が必要です。prearchive の文書登録は準備だけであり、archive/close 済みという
主張ではありません。

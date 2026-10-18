---
author: AI Cockpit maintainers
title: "WI-1087 — 有限期間の目標権限と適用範囲"
description: "P1/P2a の基盤と P2b の拒否配線。実運用の権限ソースは Unsupported。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: in_progress
authority: p2b-software-development-only
lastVerifiedBy: WI-1087-contract-authority-scope-and-expiry
---

[English](WI-1087-contract-authority-scope-and-expiry.md) · [简体中文](WI-1087-contract-authority-scope-and-expiry.zh-CN.md)

# WI-1087 — 有限期間の目標権限と適用範囲

現在は同じ WI の P2b ソフトウェア配線段階です。P1 の厳密な UTC/Clock と P2a のオフライン評価器を再利用します。実運用ソースは Unsupported で、実際の grant、信頼された有効化登録、システム権限は設定していません。全機能の受入は not_run のままです。

## Goal

検証可能な有限期間の目標権限が明示的にカバーする連続/並行WI、修復、retry、recoveryにWIごとの再承認を追加しません。旧目標の権限を新目標へ流用できません。notBefore/expiresAtは必須かつ有限で、完了/撤回/期限切れは以降の実行を止め、正当な履歴は保持します。

## Versioned design

下記の中国語版v1を規範文書とし、三言語要約は同じ設計を参照します。認証可能な出所と目標範囲、縮小のみの派生、lock/副作用前の再確認、安全停止、metadataのみの延長申請、一回の承認による適用中WIへの新期限反映、自己延長/復活の禁止、時計/identityの限界、別途移行を記述します。

- [Design v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/design-v1.zh-CN.md)
- [Acceptance v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/acceptance-v1.json)
- [Staged scope v1](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-20261016/implementation-stages-v1.json)

## Dependency and ownership

WI-1086に依存します。共有amendment/status/admission/receipt/lifecycleと台帳は直列です。二つ目のbuildや共有ファイルの並行変更を行いません。独立Clock/型モジュールの並行実装は、後日の技術レビュー、ownership確認、正常admissionが条件です。

## v2 の境界整理

v1の履歴は保持します。v2では撤回とcommitの順序、外部fencingの限界、source版の置換と主体の終了、継承期限と固定された早い上限、実行中撤回の停止証跡を明確にします。現在承認済みのソフトウェア範囲は下記のとおりです。実運用での有効化は対象外です。

- [Design v2 addendum](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/addendum-v2.zh-CN.md)
- [Acceptance v2 — planned / not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/acceptance-v2.json)

## v3

範囲縮小による適用外化と実際のactorの失効も、撤回と同じ実行中停止規則を適用します。R04はパラメータ化し、全件未実行です。

- [Design v3 clarification](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/addendum-v3.zh-CN.md)
- [R04 parameterized acceptance — not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/acceptance-delta-v3.json)

## P1 の進捗

局所 TDD とモジュール回帰は Clock 境界、厳密なモデル構造、純粋な UTC 正規化を検査します。初回のモジュール結果は core 12、protocol 12、repository 164（新しい正規化テスト 12 件を含む）が成功しました。workspace 全体の検証と CI は実際の実行後に別途記録します。全機能の受入マトリクスは not_run のままです。

実行者は `agent:Codex`、技術計画のレビュー担当は `agent:Raydot` です。`lastVerifiedBy` は本 WI の参照であり、個人の身元ではありません。[人の決定の出典](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1-activation-20261017/human-decision-source-20261017.json) に原文を保持しています。[モジュール証拠](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1/module-results.json)。P1 の元の範囲は後続段階を除外していました。今回承認済みの P2b サブ段階を下記に記します。

## P2b 配線サブ段階

任意の厳密な `governanceProfile.goalAuthority` 宣言は強制検査を要求するだけで、issuer、actor、grant、有効モードを認証しません。参照は P1 の構造と厳密な精度検査を再利用し、既定 TTL や grant を生成しません。有効化後のダウングレード防止には独立した信頼済み enrollment が必要で、編集可能な宣言だけでは保証できません。

共通の非公開検査は、リポジトリの業務読み取り、controls、amendment 回復、共有 Store/context 操作、検証ディスパッチ、CLI/MCP の業務処理より前に required 要求を拒否します。キャッシュ済み context も再検査します。静的 help/schema と MCP discovery は利用可能です。永続書き込みヘルパーはコミット直前に再検査し、履歴が固定する material recipe のバイトは外側の wrapper により保持します。

非公開 ordered-source 契約は P2a の現在事実、待機後の新鮮な Clock サンプル、元の MonotonicBudget を再利用します。既存 operation ロックを先に取得し、その後の有界 source fence が一回の永続コミットまたはディスパッチを覆います。隔離したテストアダプターで実際の linked worktree とプロセス間のコミット/撤回順序を検査します。fixture は実運用ソースの認証ではありません。ローカルロックは時間を停止せず外部の原子的コミットも保証しないため、証明できない強い外部保証は Unsupported です。

局所テストと回帰証拠は非公開で保存します。最終 Native 検証と CI は実際の候補に結合する必要があります。一時的なプロセス診断は通常の逆向きコミットで撤去し、過去の失敗/不明な試行を保持します。これは開発段階の投影であり、WI 完了や実運用の適格性ではありません。P3 の実行中停止、metadata 専用回復、source writer と、P4 の実アダプター適格性/有効化は not_run のままです。

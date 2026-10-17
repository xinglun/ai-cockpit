---
author: AI Cockpit maintainers
title: "WI-1087 — 有限期間の目標権限と適用範囲"
description: "P1 データモデルと Clock 基礎実装。有限権限の強制適用は引き続き計画段階。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: in_progress
authority: p1-software-development-only
lastVerifiedBy: WI-1087-contract-authority-scope-and-expiry
---

[English](WI-1087-contract-authority-scope-and-expiry.md) · [简体中文](WI-1087-contract-authority-scope-and-expiry.zh-CN.md)

# WI-1087 — 有限期間の目標権限と適用範囲

現在は同じ WI の P1 ソフトウェア実装段階で、原生 preflight と before-edit checkpoint を記録済みです。対象は独立モデル、テスト可能な Clock、純粋な正規化インターフェースのみです。実際の grant やシステム権限は設定していません。全機能の受入は not_run のままです。

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

v1の履歴は保持します。v2では撤回とcommitの順序、外部fencingの限界、source版の置換と主体の終了、継承期限と固定された早い上限、実行中撤回の停止証跡を明確にします。製品実装は未承認です。

- [Design v2 addendum](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/addendum-v2.zh-CN.md)
- [Acceptance v2 — planned / not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v2-20261016/acceptance-v2.json)

## v3

範囲縮小による適用外化と実際のactorの失効も、撤回と同じ実行中停止規則を適用します。R04はパラメータ化し、全件未実行です。

- [Design v3 clarification](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/addendum-v3.zh-CN.md)
- [R04 parameterized acceptance — not_run](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/design-v3-20261016/acceptance-delta-v3.json)

## P1 の進捗

局所 TDD とモジュール回帰は Clock 境界、厳密なモデル構造、純粋な UTC 正規化を検査します。初回のモジュール結果は core 12、protocol 12、repository 164（新しい正規化テスト 12 件を含む）が成功しました。workspace 全体の検証と CI は実際の実行後に別途記録します。全機能の受入マトリクスは not_run のままです。

実行者は `agent:Codex`、技術計画のレビュー担当は `agent:Raydot` です。`lastVerifiedBy` は本 WI の参照であり、個人の身元ではありません。[人の決定の出典](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1-activation-20261017/human-decision-source-20261017.json) に原文を保持しています。[モジュール証拠](../../.ai/evidence/WI-1087-contract-authority-scope-and-expiry/p1/module-results.json)。P2–P4 と実際の有効化は範囲外です。

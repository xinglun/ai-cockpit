---
author: AI Cockpit maintainers
title: "WI-1087 — 有限期間の目標権限と適用範囲"
description: "承認された目標内の継続作業と、境界を越える際の判断に関するplanned設計。"
audience:
  - maintainer
  - reviewer
workItemId: WI-1087-contract-authority-scope-and-expiry
status: planned
authority: design-and-registration-only
---

[English](WI-1087-contract-authority-scope-and-expiry.md) · [简体中文](WI-1087-contract-authority-scope-and-expiry.zh-CN.md)

# WI-1087 — 有限期間の目標権限と適用範囲

本件はplanned設計のみです。Runtime newによりplanning/not_readyとして登録し、start、checkpoint、製品実装、実際の権限有効化、merge、cleanupは行いません。製品実装authorityはmissingです。

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

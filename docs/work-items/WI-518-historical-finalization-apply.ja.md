---
author: AI Cockpit maintainers
title: "WI-518 — 歴史 finalization の適用"
description: "canonical predecessor がない場合に、PR のない歴史的 direct merge を公開 Runtime が正直に記録できるようにし、fail-closed の identity 診断を明確にする。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-518-historical-finalization-apply
lastVerifiedBy: WI-518-historical-finalization-apply
terminalArchive: .ai/work-items/archive/WI-518-historical-finalization-apply.contract.json
terminalVerification: .ai/evidence/WI-518-historical-finalization-apply.verification.json
terminalFinalization: .ai/decisions/WI-518-historical-finalization-apply.finalize.7db915ed608082f3481130460a291a4f3845908d9bd1a8e52684846f9cc9ffec.json
terminalDecision: .ai/decisions/WI-518-historical-finalization-apply.close.json
---

[English](WI-518-historical-finalization-apply.md) · [简体中文](WI-518-historical-finalization-apply.zh-CN.md)

## 目標

PR がなく canonical finalization receipt も存在しない歴史的 direct merge
（`historicalKind=direct_merge_no_pr`）に対して、監査可能で repository に束縛された適用経路を提供します。immutable history を保持し、実際の Git merge commit と parents を要求し、resource context の失敗理由を診断可能にします。

## 範囲

- Rust protocol と repository の検証/記録経路
- `finalize-recovery` CLI help
- protocol/repository 回帰テスト
- 英語・簡体字中国語・日本語の command 文書

対象 repository は read-only のままです。本 WI は歴史 receipt の書き換え、current Runtime 検証の弱体化、PR や human decision の捏造、release 公開を行いません。

## 受け入れ条件

- predecessor がない場合、完全な direct-merge receipt を `finalize-recovery` が最初の canonical record として受け付け、`finalize` と同じ archive、Contract、Git parents、repository、current Runtime 検証を実行すること
- 明示された historical low-assurance direct merge のみ provisional legacy context を解決でき、foreign worktree/base/provider binding は fail-closed で binding category を示すこと
- `finalize-recovery-plan` が決定的な identity facts と人間所有の入力項目を出力し、branch、authority、PR、decision を捏造しないこと
- 拒否された入力で immutable predecessor と repository state が変化しないこと
- 3 言語の文書が semantic/non-wire と historical-low assurance の境界を説明すること

## 検証

```text
cargo test --locked -p cockpit-protocol --test resource_finalization -- --test-threads=1
cargo test --locked -p cockpit-repository --test resource_finalization_transition -- --test-threads=1
cargo test --locked -p cockpit-cli --test resource_finalization -- --test-threads=1
cargo test --locked --workspace
```

公開 artifact の adopter acceptance は release 後の責務であり、これらの source test に置き換えません。

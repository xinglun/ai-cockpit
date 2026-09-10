---
author: AI Cockpit maintainers
title: "WI-702 — P2 IncrementalMerkle 信頼境界監査"
description: "増分コンテンツ識別計算がガバナンスへ影響する前に、metadata 再利用の境界を検証します。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-702-p2-incremental-merkle-trust-audit
status: recovered
authority: human:repository-owner
lastVerifiedBy: WI-712-wi702-finalization-recovery
recoveryDecision: .ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json
---

[English](WI-702-p2-incremental-merkle-trust-audit.md) · [简体中文](WI-702-p2-incremental-merkle-trust-audit.zh-CN.md)

# WI-702 — P2 IncrementalMerkle 信頼境界監査

## Intent and boundary

この Work Item は、将来の増分計算を検討する前に `IncrementalMerkle` の信頼境界を検証します。
North Star は Calibrated Human-Agent Trust です。repository isolation、Work Item isolation、
authorization boundary、evidence binding、recovery を cache-hit metric より優先します。

現在の repository の call-graph search では、`IncrementalMerkle` と `refresh` の呼び出しは
`crates/cockpit-git` 本体と snapshot tests にだけあります。production の governance、MCP、
doctor、status、outcome path はこの helper を構築していません。したがって、この WI は保護された
decision への影響も production performance gain も主張しません。変更は helper と focused tests
だけに限定し、他 agent の tree、branch、PR、evidence は境界外に置きます。

## Hypothesis and observed defect

変更前は cached size と mtime が一致すると read を省略していました。しかし metadata は content
identity の証明ではありません。同じ長さの置換後に mtime を戻せば、古い digest を再利用できます。
この helper を保護された decision に接続するなら、これは許容できません。

候補実装は各 refresh で declared regular file を必ず read と hash します。read の前後で metadata
を確認し、size、timestamp、消失、file type の変更を検出した場合は明示的な
`ChangedDuringRead` error を返します。public `files_reused` field は互換性のため残しますが、
常に 0 です。metadata を unchanged の証明として扱う caller はありません。

## Correctness evidence

focused `cockpit-git` suite は次をカバーします。

- 同じ長さの content を変更して元の mtime に戻すケース。
- replacement、deletion、rename、file-to-directory type change。
- path escape の拒否。
- read 中の before/after metadata の検出可能な変更を `ChangedDuringRead` で拒否するケース。

base revision の旧 test は unchanged file の reuse を明示的に期待していました。candidate test
は unchanged file を再読し、same-length/restored-mtime で Merkle root が変わることを検証します。
これは信頼性の修正であり、performance optimization ではありません。将来 production caller が
追加されればコストが増える可能性があります。

## Limits and governance state

metadata guard は read 中に bytes と全ての観測可能 metadata を変更して元に戻す concurrent edit
を証明できません。これは明示的な validity limitation であり、cache hit や correctness claim
ではありません。filesystem notification、persistent index、layered Merkle tree、production
integration、release behavior、performance benchmark は scope 外です。

この文書は前置の historical record として、単独の green terminal outcome を主張しません。WI-712 が
append-only recovery、review 済み PR の reconciliation、finalization、close、documentation promotion
を完了しており、関連記録は下の recovery boundary にあります。前置 bytes は immutable のままで、
この Work Item は引き続き production performance benefit を主張しません。

## Merge 後の recovery boundary

PR #700 は repository owner の review 後に merge され、merge commit は
`bd00a7ce888c2d0dba012da21ba1616eeeab0014`、reviewed head は
`4eedf23ddf1e4a0491fb978127d61d852e6a5a1f` です。この Work Item の immutable な
pre-merge finalization root は `86f8535f` を束縛しています。途中の range には
pending parity registry の変更も含まれるため、installed Runtime はこの range を
append-only finalization transition として扱うことを正しく拒否します。

WI-702 の archive、verification、Outcome、Events、Contract、finalization bytes は
historical evidence として保持します。append-only recovery decision は
`.ai/decisions/WI-702-p2-incremental-merkle-trust-audit.recovery.json` であり、
新しい merge 後の finalization、parity 登録、正確な cleanup boundary は WI-712 が
担当します。この recovery は Calibrated Human-Agent Trust を維持し、performance
benefit を主張しません。

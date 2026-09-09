---
author: AI Cockpit maintainers
title: "WI-680 — P1 協作場景行列"
description: "状態と遷移から生成した協作場景行列、および reference-parity 台帳への WI-679/WI-680 の三言語登録。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-680-p1-collaboration-scenario-matrix
status: in_progress
authority: authorized
lastVerifiedBy: WI-680-p1-collaboration-scenario-matrix
---

[English](WI-680-p1-collaboration-scenario-matrix.md) · [简体中文](WI-680-p1-collaboration-scenario-matrix.zh-CN.md)

# WI-680 — P1 協作場景行列

## 意図

AI Cockpit 協作言語専項の P1 範囲を推進する:本リポジトリが実際にサポート
するライフサイクル、証拠状態、授権状態、操作種別から場景行列を生成し、
網羅よりも重要な境界と混同しやすい組み合わせを優先する。あわせて、WI-679
が「アーカイブ前」の正しいコミット順序では完了できず、かつ既に push 済みの
履歴を書き換えたくなかったために残した reference-parity 台帳登録を完了す
る。

## 境界

これは文書のみの Work Item である。
`docs/reference/collaboration-scenario-matrix.md`(+ zh-CN/ja)、
`docs/reference/collaboration-scenario-matrix.json`(構造化された事実の出
典)、`docs/reference/README.md`(+ zh-CN/ja)への索引エントリ1件、
`docs/work-items/WI-679-*` と `docs/work-items/WI-680-*` の記録ページ、
そして `docs/reference/reference-parity.*` への WI-679・WI-680 の登録行を
追加する。CLI、MCP、Contract スキーマ、Outcome スキーマ、ライフサイクルの
挙動はいずれも変更しない。場景を制御されたテストリポジトリに対する自動化
された実行可能チェックへ変換することは明示的に対象外であり、納品文書内で
後続作業として明記する。

## 授権記録

2026-09-08、Work Item プロセスを通じて協作言語専項を継続すること、および
納品過程で発見された文書登録上の欠落を修復することについて、人間から明示
的な授権を得た。正確な範囲と証拠は Contract と PR に記録されている。

## 受け入れとライフサイクル

- 場景行列の構造化 JSON は必須カテゴリすべてを網羅し、各エントリに
  sourceType(observed/documented/designed)を付し、その大半は本納品中に
  得られた実際のコマンド/出力を引用している。
- WI-679 と WI-680 はいずれも「アーカイブ前」の
  `進行中 → verified close 後 Implemented` ステータスで三言語の
  reference-parity 台帳に現れ、各行の初出は本 Work Item 自身の verification
  証拠コミットに先行する。
- `start → preflight → checkpoint → verify → finish → archive → close` が統治
  された経路であり、`user_visible_benefit_not_declared` は明示されたまま
  である。
- 精査対象の head 上で `bash tests/docs/documentation_acceptance.sh` が合格
  する。

## 証拠

- archive: `.ai/work-items/archive/WI-680-p1-collaboration-scenario-matrix.contract.json`
- verification: `.ai/evidence/WI-680-p1-collaboration-scenario-matrix.verification.json`
- finalization: `.ai/decisions/WI-680-p1-collaboration-scenario-matrix.finalize.json`(合併後に生成予定)
- close: `.ai/decisions/WI-680-p1-collaboration-scenario-matrix.close.json`(合併後に生成予定)

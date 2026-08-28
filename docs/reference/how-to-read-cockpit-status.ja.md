---
title: Cockpit Status の読み方
description: 生成された status と Outcome を、範囲を限定した人の判断に変換する。
author: AI Cockpit maintainers
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/how-to-read-cockpit-status.md
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - human_outcome_handoff
---

# Cockpit Status の読み方

[English](how-to-read-cockpit-status.md) · [简体中文](how-to-read-cockpit-status.zh-CN.md) · [日本語](how-to-read-cockpit-status.ja.md)

このページは、技術者でない承認者を含む Work Item の確認者向けです。生成された事実を範囲付きの判断へ
変換する読み方を示しますが、判断そのものを代行しません。

## 最初に人向け handoff を読む

まず repository-bound な status を確認し、その後に人向け handoff を再表示します。

```sh
ai-cockpit status --repo <repository> --id <work-item>
ai-cockpit work-item outcome --repo <repository> --id <work-item>
```

2 つ目の command は `Outcome: 🔴`、`Outcome: 🟡`、`Outcome: 🟢` のいずれかで始まります。
CLI と repository-bound MCP の `work_item_outcome` は同じ human-facing projection を返します。
表示方法は host が決められますが、折りたたまれた log や raw `work_item_get` は handoff の代わりになりません。
機械が安定した `OutcomeV2` を必要とする場合だけ `--json` を使います。

## この順序で読む

1. **Task Result と marker** — Work Item と判断 signal を確認します。
2. **完了したこと** — Runtime summary と delivered-change の記録を読みます。
3. **発見された問題と停止** — failed gate または停止理由を確認します。
4. **解決した問題と risk** — 解決済みの記録と残存 risk/warning を分けます。
5. **不明点** — unknown は未解決の質問であり、隠れた pass ではありません。
6. **人間の判断** — 記録がある場合は actor、authority source、reason、evidence/policy ref、時刻、再開条件を確認します。
7. **検証と evidence** — 行動する前に列挙された repository-bound receipt と freshness を確認します。
8. **影響と次の action** — 未宣言の user benefit は unknown のままです。次の action は recovery/review condition に従います。
9. **Acceptance criteria (Contract language)** — owner の原文 Contract を読みます。監査可能性のため変更しません。

Reference source の `Key Conclusion`、`Recommendation`、`Decision Drivers`、`Evidence`、
`Scenario Coverage` という読み方は保持します。Rust Runtime は typed Outcome section と別の status projection を使うため、
これは semantic parity であり source JSON wire contract の互換性ではありません。

## 色の意味

| Marker | 意味 | 安全な action |
| --- | --- | --- |
| 🟢 Green | current で identity-bound な evidence が review に十分です。 | 列挙された evidence を確認し、組織の手順で必要な判断を取得します。merge/release の承認ではありません。 |
| 🟡 Yellow | evidence が不足・部分的・履歴であるか、人の判断が残っています。 | 調査、evidence の収集、または明示的な判断の記録を行い、安全な状態を保ちます。 |
| 🔴 Red | mandatory control が失敗、権限/scope が無効、または evidence が矛盾・破損しています。 | 停止し、記載された recovery condition に従います。推測や generated record の手編集は禁止です。 |
| `unknown` field | 事実または projection を信頼できない、または未宣言です。 | clarification または fresh な bound receipt を求めます。green にはなりません。 |

色は score ではなく semantic signal です。Green Outcome は current evidence を review できることだけを示し、
merge、release、publication、security claim、enterprise assurance を承認しません。Yellow/Red は無関係な command の再実行だけでは直りません。

## 停止条件と evidence boundary

stale、malformed、symlink、別 Work Item、別 repository、snapshot mismatch の status/evidence は Runtime で再生成します。
色を変えるために生成された Contract、Summary、Outcome、receipt、archive、decision を編集してはいけません。
履歴 evidence は immutable のまま保持し、current result が必要な場合は current Runtime で再検証します。

local verification、Hosted CI、provider attestation、SBOM/provenance、enterprise approval は別々の evidence boundary です。
各 receipt の出所を report に表示し、local green を provider/enterprise assurance と呼び替えません。

## 言語と adopter への継承

Runtime が生成する見出し、marker、status、summary、unknown code、recovery hint は `AI_COCKPIT_LANGUAGE`（または adapter の言語）に従います。
Contract の intent、scope、acceptance criteria は原文のままにし、自動翻訳で governance の事実を変えません。
Agent conversation は利用者の言語で handoff を表示し、原文 Contract も残します。

各 adopter repository は明示的な `--repo` で同じ route を利用します。共有 Runtime に current project や global Work Item はないため、
ある repository の status が別 repository の作業を承認・説明することはありません。

## 次に読むページ

- [Governance Profile](governance-profiles.ja.md) — リスクに応じた quality routing。
- [人間向け Outcome](outcome-report.ja.md) — handoff と machine boundary の定義。
- [Command reference](commands.ja.md) — lifecycle command と明示的な binding。
- [Troubleshooting と recovery](troubleshooting.ja.md) — 停止後の復旧。

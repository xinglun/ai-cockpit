---
author: AI Cockpit maintainers
title: Governance Profile
description: Light、Standard、Strict Work Item のリスクベース品質ルーティング。
audience:
  - adopter
  - maintainer
  - reviewer
status: implemented
authority: translation
canonical: docs/reference/governance-profiles.md
lastVerifiedBy: WI-346-reference-governance-profiles-status
capabilityClaims:
  - risk_based_quality_routing
---

# Governance Profile

[English](governance-profiles.md) · [简体中文](governance-profiles.zh-CN.md) · [日本語](governance-profiles.ja.md)

AI Cockpit は Repository の事実、Work Item Contract、実行 stage、適用 policy から quality route を選びます。
route の強度は `light < standard < strict` です。変更が混在する場合は最も高い route を使い、
unknown または空の path evidence によって route を弱めることはありません。

ここで説明するのは verification の強度であり、assurance の保証や人の権限を代替するものではありません。

## 3 つの Profile

| Profile | 代表的な変更 | Target route |
| --- | --- | --- |
| `light` | 文書、コメント、非実行 example、format のみ | focused quality check |
| `standard` | 通常の source、test、bug fix、小規模 refactor | project verification と reference-impact check |
| `strict` | governance、CI、installer、security、dependency、破壊的/Public API、migration、calibration、evidence Schema | repository と supply-chain の full check |

`release` は第 4 の Profile ではなく operation class です。release resource を扱う operation は
strict の下限に release-preflight、artifact、checksum、SBOM、provenance、adopter check を追加できます。
non-release の strict 変更が、名前だけを理由に release graph を得ることはありません。

## Profile の効果と assurance

次の次元は分離して扱います。

- `VerificationTier`（`T0`–`T3`）は必要な verification の強さです。
- `EvidenceAssurance`（`SelfDeclared`、`RepositoryVerified`、`ProviderVerified`、
  `EnterpriseVerified`）は evidence を裏付ける主体または仕組みです。
- cost と reuse の観測は資源使用量の advisory 情報であり、要求を下げたり unknown を green にしたりしません。

`T3` は `ProviderVerified` を意味せず、`strict` は `EnterpriseVerified` を意味しません。
Tier/assurance の要求は Organization Policy、Project Policy、Release Policy、protected gate、
または人が所有する Contract まで追跡できなければなりません。Planner は escalation を提案できますが、
policy を plan の内部に隠してはいけません。

すべての route は scope、trust、lifecycle、evidence integrity という同じ mandatory control floor を維持します。
optional な heavy/cost check は authorization や security の switch ではありません。unknown profile、壊れた policy、
unsafe path、無効な base、不完全な override、mandatory control の削除は fail closed です。

## Route の選択

Repository-bound route は、保護対象の command を実行する前に評価されます。

```text
repository snapshot + Contract + stage/policy
                 ↓
        `ai-cockpit gate --repo <path> --contract <file>`
                 ↓
      宣言された verification command / hosted gate
```

Contract base から見た committed、staged、unstaged、untracked path を評価します。
生成される receipt は repository、Work Item、base/snapshot、selected profile、Verification Tier、
assurance requirement、理由、gate identity に bind されます。Receipt は route evidence であり authorization token ではありません。

明示した Profile は自動結果を上げることだけができ、下げることはできません。downgrade には、
期限付きかつ現在の Work Item に限定された human override、approval evidence、理由、認識した risk、
実行しない check の一覧が必要です。永続的な例外は作成しません。

## Session と repository の境界

Quality report writer は worktree-local の non-blocking lock を使います。同じ worktree の 2 回目の呼び出しは
fail closed になり、別 worktree は並列実行できます。共有 Runtime に current project や global active Work Item はありません。
各 adopter repository は明示的な `--repo` を渡し、Contract、evidence、adapter record を repository ごとに隔離します。

参考 template の `make ai-cockpit-quality` と Python router は conformance 用の資料であり、この Rust repository が
コピーする command ではありません。対象の境界は、インストール済み Runtime、明示的な repository context、
typed Contract/verification record、repository が宣言する CI gate です。local result を Hosted または enterprise assurance に自動昇格しません。

## 結果の安全な読み方

人向けの handoff は `ai-cockpit work-item outcome --repo <path> --id <work-item>` で読みます。
Green は列挙された evidence を review できることを示すだけで、merge、release、publication、security claim の承認ではありません。
Yellow は evidence または判断が不完全、Red は mandatory control または context が無効で停止が必要です。
[Cockpit Status の読み方](how-to-read-cockpit-status.ja.md)も参照してください。


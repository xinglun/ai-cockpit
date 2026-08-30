---
author: AI Cockpit maintainers
title: “TypeScript Web fixture の適応境界”
description: “固定した TypeScript Web fixture をファイル単位で Rust-native に対応づけ、application、npm toolchain、lifecycle script はコピーしません。”
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: current
authority: canonical
lastVerifiedBy: WI-432-reference-typescript-fixture-boundary
sourceCommit: e5acb677da6621004d96f0ef353c58fe8d3acfbf
capabilityClaims:
  - semantic_reference_mapping
---

# TypeScript Web fixture の適応境界

このページは pinned reference fixture `examples/fixtures/typescript-web/` の 11 ファイルを
一つずつ比較します。TypeScript/Web adopter に有用な意味を保ちますが、fixture application、
npm dependency、Node command、source lifecycle implementation を Rust Runtime にコピーしません。

[English](typescript-fixture-adaptation.md) · [简体中文](typescript-fixture-adaptation.zh-CN.md) · [日本語](typescript-fixture-adaptation.ja.md)

## ファイル単位の対応

| 固定 source file | source の事実 | Rust-native の対応と境界 |
| --- | --- | --- |
| `.gitignore` | `node_modules/`、`dist/`、生成される `.fixture-state.json` を除外します。 | build output の hygiene は adopter の責任です。release harness は独自の隔離 root を使い、この ignore file を生成・コピーしません。 |
| `evidence.json` | local npm lifecycle evidence を説明し、provider evidence が unavailable であることを明記します。 | Runtime verification は repository、snapshot、Runtime、command、result identity を bind します。source-local evidence を provider、hosted CI、sandbox、immutable audit、enterprise evidence に昇格しません。 |
| `fixture.json` | TypeScript Web stack、Node/npm/TypeScript toolchain、platform、safe path、test path を宣言します。 | Project Observer/Profile は確認済み adopter facts を記録できます。Runtime は fixture metadata から capability、platform readiness、Contract scope を推論しません。 |
| `package-lock.json` | TypeScript 5.8.3 npm dependency と registry integrity を固定します。 | dependency manifest と registry は adopter の責任です。shared Runtime は Node package を install せず、この lockfile を同梱せず、Runtime supply-chain evidence とみなしません。 |
| `package.json` | build、test、lint、format-check、lifecycle の npm script を定義します。 | adopter は Contract に明示的な verification argv を定義します。Runtime は各 result を記録し、governance lifecycle（`preflight` から `close`）を npm orchestration と分離します。 |
| `scripts/format-check.mjs` | `src/index.ts` の末尾 newline と tab の有無を確認します。 | fixture 固有の format rule です。adopter は自分の formatter command を宣言し、local result として bind します。 |
| `scripts/lifecycle.mjs` | install/configure/normal、ambiguous/critical-domain の block、upgrade/rollback、release check を実行します。 | installed Runtime が repository-bound lifecycle、human review pause、evidence binding、recovery、visible Outcome を提供します。source Node script を Runtime authority として実行・コピーしません。 |
| `scripts/lint.mjs` | sample source に `evaluateRequest` があり `any` がないことを確認します。 | application 固有の lint であり portable governance control ではありません。lint command と acceptance evidence は adopter が所有します。 |
| `src/index.ts` | sample request evaluator が `allow`/`block`、reason、resume condition を返します。 | application behavior は adopter-owned です。Runtime の decision と stop state は typed governance record であり、sample policy を import・推論しません。 |
| `test/index.test.mjs` | Node test が normal request の allow と dangerous request の block を assert します。 | adopter が自分の test command を提供・実行します。source fixture assertion を Runtime、provider、enterprise evidence に昇格しません。 |
| `tsconfig.json` | strict TypeScript、NodeNext module、declaration output を有効にします。 | TypeScript compiler configuration は adopter の責任です。Runtime は明示された command result を受け取りますが、Node/TypeScript toolchain や compiler setting を保証・コピーしません。 |

## Installation と adopter 境界

fixture の stack metadata は AI Cockpit の installation recipe ではありません。adopter の外側に
shared Runtime を一つ install し、repository を明示的に attach します。

```bash
repo=/path/to/typescript-repository
ai-cockpit attach --repo "$repo"
ai-cockpit inspect --repo "$repo"
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
```

Node.js、npm、TypeScript、dependency lock、build output、hosted/provider evidence は adopter が
所有します。その後の Runtime command には同じ `--repo` を必ず付け、Contract scope、profile、
snapshot、evidence、knowledge、Agent adapter record を repository ごとに隔離します。

## Adopter が継承するもの

attach 済み TypeScript/Web project は、shared Runtime の Contract validation、unknown の
fail-closed、identity-bound evidence、lifecycle、visible human Outcome を継承します。fixture の
Node dependency、npm script、application code、test、または command 実行済みという claim は継承しません。
外部 authority が提供しない限り、local npm result は provider、hosted CI、Release、enterprise
evidence ではありません。

これは semantic/documentation parity であり、TypeScript toolchain support、source command
compatibility、JSON-wire compatibility ではありません。Second-technology adopter acceptance は
別途認可された post-release Work Item とします。

[Reference index](README.ja.md) · [Reference file comparison](reference-file-comparison.ja.md)

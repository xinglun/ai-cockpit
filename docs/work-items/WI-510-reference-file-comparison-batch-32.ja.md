---
author: AI Cockpit maintainers
title: "WI-510 — installer entrypoint と wizard locale の境界"
description: "source installer/wizard 実装を copy せず、保守対象 4 file を一つずつ比較する。"
audience:
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
workItemId: WI-510-reference-file-comparison-batch-32
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-510-reference-file-comparison-batch-32
terminalArchive: .ai/work-items/archive/WI-510-reference-file-comparison-batch-32.contract.json
terminalVerification: .ai/evidence/WI-510-reference-file-comparison-batch-32.verification.json
terminalFinalization: .ai/decisions/WI-510-reference-file-comparison-batch-32.finalize.json
terminalDecision: .ai/decisions/WI-510-reference-file-comparison-batch-32.close.json
---

[English](WI-510-reference-file-comparison-batch-32.md) · [简体中文](WI-510-reference-file-comparison-batch-32.zh-CN.md)

## Goal

固定した reference commit の `install.sh` と English/Japanese/Simplified Chinese の wizard locale を一つずつ読み、各 path の evidence-backed semantic decision と Rust counterpart boundary を記録します。これは比較と境界定義であり、source Shell/Python installer、wizard、locale bytes、source JSON wire shape は copy しません。

## File-by-file decisions

| Pinned path と source digest | Classification | Target boundary |
| --- | --- | --- |
| `install.sh` — `sha256:14f157f828e3ba8d1dd0886708b7eae223fe6d08` | implemented-different-by-design | Rust immutable public Release、checksum/SBOM/provenance、explicit repository attach、isolated adopter acceptance が source selection、verification、cleanup、rollback、isolation を担います。source Shell/Python installer や implicit target write は取り込みません。 |
| `locales/wizard/en.json` — `sha256:1b9bfc3535e507c8478b071b641d974cb031e59e` | reference-only | Rust English Runtime label と human Outcome は installation/command/Outcome reference に記録します。interactive wizard の prompt/session control は host/Agent adapter UX です。 |
| `locales/wizard/ja.json` — `sha256:8fab9ba89bd2bac5ccd51e8cb70dfea719435f5c` | reference-only | Rust Japanese Runtime presentation を記録します。第二の interactive installer は提供せず、locale text は repository change を authorize しません。 |
| `locales/wizard/zh-CN.json` — `sha256:591e11709864edf2846bfe63aab246b1dafd6473` | reference-only | Rust Chinese Runtime presentation を記録します。source wizard bytes は copy せず、locale は repository change を authorize しません。 |

## Object/adopter inheritance boundary

各 object/adopter repository は shared Runtime を外部に一度だけ install し、明示的な `--repo` で自分の repository context を bind します。継承するのは repository-local な `attach`、Agent adapter、Contract、evidence、knowledge、human Outcome boundary です。source installer 実装、stack 固有 wizard、source locale JSON、provider decision は継承しません。Contract fact は authoring language に保持し、Runtime-owned presentation のみ localize します。

## Acceptance criteria

- 4 pinned path に source digest、reason、counterpart list がある。
- installer semantic は Rust Release/distribution と adopter documentation で表現し、source code は copy しない。
- locale は reference-only のまま、Runtime multilingual presentation と adapter responsibility を明示する。
- English/Chinese/Japanese の inventory、comparison、parity、本 Work Item documentation が同期し、`migrate-gap` がない。
- conformance、documentation、workspace verification が通り、object repository、global Agent/MCP configuration、無関係な Runtime behavior を変更しない。

## Verification

Contract に宣言した check は次のとおりです。

```text
python3 tests/conformance/reference_file_inventory.py --check
bash tests/conformance/reference_file_inventory_test.sh
python3 tests/conformance/reference_inventory_docs_test.py
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
cargo test --locked --workspace
```

reference checkout は比較専用であり、source installer、locale、その他 source file は target repository に追加しません。

## Terminal evidence

front matter に示す生成 archive、verification、finalization、close receipt が lifecycle status の authority です。comparison page に同じ 4 decisions と inventory count を記録し、historical evidence は書き換えません。

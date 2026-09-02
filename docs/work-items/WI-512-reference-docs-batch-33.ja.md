---
author: AI Cockpit maintainers
title: "WI-512 — governance reference documentation batch 33"
description: "Governance と verification boundary の reference page を file 単位で比較する bounded batch。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: human-authorized
workItemId: WI-512-reference-docs-batch-33
sourceCommit: fde3380f81fea5fd2e288f7a8849f737dc074060
lastVerifiedBy: WI-512-reference-docs-batch-33
terminalArchive: .ai/work-items/archive/WI-512-reference-docs-batch-33.contract.json
terminalVerification: .ai/evidence/WI-512-reference-docs-batch-33.verification.json
terminalFinalization: .ai/decisions/WI-512-reference-docs-batch-33.finalize.json
terminalDecision: .ai/decisions/WI-512-reference-docs-batch-33.close.json
---

[English](WI-512-reference-docs-batch-33.md) · [简体中文](WI-512-reference-docs-batch-33.zh-CN.md)

## Goal

固定した local reference commit の governance/verification page を一つずつ読み、Rust-native counterpart を記録します。意味は保ちますが、source Python、Shell、Make、provider、installer、wizard、JSON wire は copy しません。

## File-by-file decision

| Pinned source path / digest | Classification | Rust counterpart / non-claim |
| --- | --- | --- |
| `docs/reference/schemas.md` — `4ed6c44bfcfea93300c39fa467170902932e4371d218f09269bed9da26fbf625` | implemented-different-by-design | `docs/reference/schemas.*`、typed Protocol、schema test。source registry は wire requirement ではありません。 |
| `docs/reference/test-architecture.md` — `3c475a84e6b7634c6d98c44af029d6b01aff6a36da5649195d1daa1d52d2a82f` | implemented-different-by-design | tri-language test architecture、dynamic quality route、governance gate。tier は assurance ではありません。 |
| `docs/reference/test-weakening-guard.md` — `17824614224f43bde778ab3985d1abc42d6c53ad0b5a5a26d3fc371e25a3ba7c` | implemented-different-by-design | Rust governance signal/regression と tri-language guard page。source implementation は copy しません。 |
| `docs/reference/test-weakening-guard.zh-CN.md` — `9b5b06cc25f0e05443a3b5b9181b3a04c076a9e67c5d8f17c02f2f45412f548e` | implemented-different-by-design | 同じ typed guard boundary の Chinese presentation。locale は authority を与えません。 |
| `docs/reference/test-weakening-guard.ja.md` — `0ba5c1fd600990111a1942dcd48e4e4bda5903f6119fc51bbd67f0ffd7702b76` | implemented-different-by-design | 同じ typed guard boundary の Japanese presentation。locale は authority を与えません。 |
| `docs/reference/verification-fixture-boundary.md` — `712ecf6a4aed8793464b40ac41cb8b9d19a47663da5c28b61403e14552990f1e` | implemented-different-by-design | tri-language fixture boundary と isolation/adopter manifest。source helper bytes は copy しません。 |
| `docs/reference/troubleshooting.md` — `57f2415177d9135c506ef9c325dd7dc8bb989ee4801907da173bac5df640dee3` | implemented-different-by-design (WI-504 revalidated) | explicit `--repo` Runtime recovery。provider wizard/toolchain は外部責任です。 |
| `docs/reference/troubleshooting.ja.md` — `0addca04e66d0118311cf7a169b8dd060d42b500c265f85496b014300749bbf9` | implemented-different-by-design | Japanese recovery と adapter boundary。source session implementation は copy しません。 |
| `docs/reference/upgrade.md` — `3ebbb05b52a281c1974dc446e6707fc8cbd5f3fddd2897f6c8bf868133ac92f4` | implemented-different-by-design | shared Runtime upgrade、explicit repository migration、immutable evidence。 |
| `docs/reference/upgrade.ja.md` — `48367289304c82e14a7ace646092b6f115b1c18b4ee01ab96122f41255ec01e9` | implemented-different-by-design | Japanese upgrade/migration presentation。source installer と locale JSON は copy しません。 |
| `docs/reference/work-item-lifecycle-closure.md` — `91fab8d045cd45eeb616d768e174ba840b5c5f7ba1a0a4a819065515822ad324` | implemented-different-by-design (WI-504 revalidated) | Rust finalize/close、recovery、`ready_on_base`。source Make/Python recovery は Rust command ではありません。 |
| `docs/reference/work-item-lifecycle-closure.ja.md` — `e47bdb794178855b2f4dad4b40fc6d5ee4f150d1c2c58a3f69eb897835593ea3` | implemented-different-by-design | Japanese closure と historical recovery boundary。provider route は外部責任です。 |

Target と adopter は shared external Runtime、isolated repository context、Contract/evidence/knowledge record、Agent adapter boundary、human Outcome handoff を継承します。source-specific command、provider policy、generated history、wire shape は継承しません。WI-504 の 2 path は再確認のみで重複登録せず、残り 10 path に WI-512 record があります。

## Acceptance / verification

- 各 row は source commit `fde3380f81fea5fd2e288f7a8849f737dc074060` で確認しました。
- すべてに classification、counterpart、明示的 non-claim があり、この batch に `migrate-gap` や deferred はありません。
- ledger は 5,119 path、669 retired path の append-only history を保持し、reference/object repository は変更しません。
- tri-language comparison、parity、documentation、inventory、workspace check を Contract の宣言どおり実行し、terminal evidence に記録します。

これは semantic/documentation parity であり、source command、Python module、JSON-wire compatibility ではありません。

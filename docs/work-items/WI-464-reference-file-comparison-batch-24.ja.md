---
author: AI Cockpit maintainers
title: "WI-464 — workflow/build rebaseline"
description: "source 変更 4 path を Rust-native CI/release boundary と再比較します。"
audience:
  - maintainer
  - reviewer
status: current
authority: canonical
---

# WI-464 — workflow/build rebaseline

この Work Item は、以前の workflow 比較後に source bytes が変わった 4 path を再読します。
source checkout は local pinned reference commit
`fde3380f81fea5fd2e288f7a8849f737dc074060` であり、copy する実装ではありません。

| Pinned source path | Classification | Rust-native decision |
| --- | --- | --- |
| `.github/workflows/compatibility.yml` | implemented-different-by-design | source の ShellCheck install と Python/multi-stack matrix は source/provider boundary。Rust は pinned-action policy、dynamic quality route、Rust workspace/platform check、public adopter acceptance を持ちます。 |
| `.github/workflows/release.yml` | implemented-different-by-design | source の `release-digests.json` archive projection と obsolete `release.json` dual-asset check の削除は、Rust の release manifest/`SHA256SUMS`、SBOM/provenance、platform smoke、adopter evidence に対応します。source projection bytes は copy しません。 |
| `.github/workflows/smoke.yml` | implemented-different-by-design | source は `REPORT_LANGUAGE` Make argument を削除しました。Rust に source `smoke.yml` はなく、CI/release/gate manifest/immutable adopter harness が explicit repository context で bounded check を分担します。 |
| `Makefile` | implemented-different-by-design | source Python/Make shard、knowledge、language helper は source-only。Rust は Cargo、CLI、canonical gate manifest、explicit `--repo` を使用し、第二の Make governance layer は持ちません。 |

この rebaseline では Rust 実装の omission は見つかりませんでした。target の action pin は
target 自身の reviewed action-runtime policy が管理し、source matrix の pin を Rust route に
暗黙に置き換えません。

Machine ledger は 4 path をこの Work Item に記録し、
`sourceChangedSincePrevious` provenance を保持しながら deferred を解消します。
これは semantic/documentation parity であり、source file、Python/Make、provider、JSON-wire
compatibility ではありません。object/adopter repository は source workflow file ではなく、
shared Rust Runtime と repository-local evidence boundary を継承します。

## Verification

- `python3 tests/conformance/reference_file_inventory.py --check`
- `bash tests/conformance/reference_file_inventory_test.sh`
- この Work Item が宣言した documentation/repository gate checks


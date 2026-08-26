---
author: AI Cockpit maintainers
title: "WI-304 — reference workflow 比較 batch 02"
workItemId: WI-304-reference-file-comparison-batch-02
description: "pinned reference の次の二つの workflow を file 単位で比較し、source tooling を copy せず Rust-native と external/adopter boundary を記録します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-304-reference-file-comparison-batch-02
terminalArchive: .ai/work-items/archive/WI-304-reference-file-comparison-batch-02.contract.json
terminalVerification: .ai/evidence/WI-304-reference-file-comparison-batch-02.verification.json
terminalFinalization: .ai/decisions/WI-304-reference-file-comparison-batch-02.finalize.453c648a442f9cff7ada6d294032a3a0a4043b669d0be65fa1afca407a3b49cf.json
terminalDecision: .ai/decisions/WI-304-reference-file-comparison-batch-02.close.json
authority: canonical
---

# WI-304 — reference workflow 比較 batch 02

## Intent と goal

次の deferred reference file、`.github/workflows/compatibility.yml` と
`.github/workflows/smoke.yml` を一つずつ比較します。trigger、matrix、dependency、artifact、
release/measurement condition、installer responsibility をすべて記録し、Rust-native
counterpart または明示的な external/adopter boundary に結び付けます。source の Python、
Make、installer、workflow byte は copy しません。

## Scope と boundary

Scope は reference inventory generator/regression ledger、三言語の比較 page、そしてこの
Work Item の三言語 projection です。既存の inventory、documentation、workspace check は
実行できますが、Runtime semantics は変更しません。

Out of scope は reference Python module、Make target、`install.sh`、multi-stack fixture の
copy、full multi-language/mobile compatibility matrix や second-technology adopter の実装、
`crates/**`、Runtime semantics、global Agent/MCP configuration、release、immutable history
evidence の変更です。

## Pinned source と比較 facts

- Reference: `spirex-ds-dev/ai-cockpit-template` commit
  `e5acb677da6621004d96f0ef353c58fe8d3acfbf`。
- Rust comparison ledger baseline: target commit
  `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`。
- Installed Runtime: `ai-cockpit 0.2.33`、binary SHA256
  `sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`。
- Source workflow は compatibility の八つの責任（ShellCheck、Python platform、lockfile
  reproducibility、real/extended/mobile matrix、latest probe、二つの aggregate gate）と、
  smoke の project shard、installation/release/measurement path、artifact、最終 CI receipt を持ちます。
- Target boundary は `ci.yml`、`release.yml`、canonical gate manifest、immutable public/N-1
  adopter harness です。adopter toolchain と source-specific installer/multi-stack test は
  external または adopter-owned です。

## Acceptance criteria

1. 二つの pinned workflow を trigger、permission、concurrency、全 job/matrix、`needs`、input、
   artifact path、blocking condition、release/measurement branch、installer command まで比較します。
2. 全 source responsibility に Rust counterpart または明示的な external/adopter/deferred
   boundary があり、暗黙の parity claim をしません。
3. ledger はこの二つだけを WI-302 の deferred 集合から WI-304 へ移し、non-empty reason と
   counterpart evidence を持ち、unclassified record を残しません。
4. 三言語の比較 page と Work Item projection が同じ semantic/non-wire boundary を示し、target
   equivalent のない Python/Make/installer responsibility を external として保持します。
5. dynamic `light`/`standard`/`strict` route、明示的 `--repo`、shared Runtime、isolated adopter
   evidence は変更しません。
6. inventory、documentation、governance、workspace check が通り、installed Runtime の lifecycle
   と reviewed PR を完了します。

## Known boundary

Source ShellCheck job は source-only `install.sh` を検査します。Target に installer はなく、
現在は shell syntax validation を行います。Target script の ShellCheck policy は別の CI hygiene
decision であり、source installer を copy したり source matrix が Runtime 内で動くと主張したりしません。

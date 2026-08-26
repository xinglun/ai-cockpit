---
author: AI Cockpit maintainers
title: "WI-304 — reference workflow comparison batch 02"
workItemId: WI-304-reference-file-comparison-batch-02
description: "Compare the next two pinned reference workflows file by file and record Rust-native and external/adopter boundaries without copying source tooling."
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

# WI-304 — reference workflow comparison batch 02

## Intent and goal

Compare the next two deferred reference files, `.github/workflows/compatibility.yml`
and `.github/workflows/smoke.yml`, against the Rust repository one file at a time.
Record every trigger, matrix, dependency, artifact, release/measurement condition,
and installer responsibility, then map it to a Rust-native counterpart or an
explicit external/adopter boundary. This Work Item introduces no source Python,
Make, installer, or workflow-byte copy.

## Scope and boundary

In scope: the reference inventory generator and regression ledger; the English,
Chinese, and Japanese comparison pages; and these three-language Work Item
projections. The comparison may execute existing inventory, documentation, and
workspace checks but does not change Runtime semantics.

Out of scope: copying reference Python modules, Make targets, `install.sh`, or
multi-stack fixtures; implementing a full multi-language/mobile compatibility
matrix or second-technology adopter; changing `crates/**`, Runtime semantics,
global Agent/MCP configuration, release version/publication, or immutable
historical evidence.

## Pinned sources and comparison facts

- Reference repository: `spirex-ds-dev/ai-cockpit-template` at
  `e5acb677da6621004d96f0ef353c58fe8d3acfbf`.
- Rust comparison ledger baseline: target commit
  `a533d49dfa848d95742833f8cd1b5f7e1bb897d5`.
- Installed Runtime: `ai-cockpit 0.2.33`, binary SHA256
  `sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4`.
- Source workflows: eight compatibility responsibilities (ShellCheck, Python
  platform, lockfile reproducibility, real/extended/mobile matrices, latest
  probe, and two aggregate gates) and the smoke workflow's project shards,
  installation/release/measurement paths, artifacts, and final CI receipt.
- Target boundary: `ci.yml`, `release.yml`, the canonical gate manifest, and
  immutable public/N-1 adopter harnesses provide Rust product/release evidence;
  adopter toolchains and source-specific installer/multi-stack tests remain
  external or adopter-owned.

## Acceptance criteria

1. The two pinned workflow files are compared completely, including triggers,
   permissions, concurrency, every job/matrix, `needs`, inputs, artifact paths,
   blocking conditions, release/measurement branches, and installer commands.
2. Every source responsibility has a cited Rust counterpart or an explicit
   external/adopter/deferred boundary; no silent parity claim is made.
3. The ledger moves exactly these two records from WI-302's deferred set to
   WI-304 with non-empty reason and counterpart evidence, and has no unclassified
   record.
4. The three language comparison pages and Work Item projections state the same
   semantic/non-wire boundary and preserve the source's Python/Make/installer
   responsibilities as external where no target equivalent exists.
5. Existing dynamic `light`/`standard`/`strict` routing, explicit `--repo`,
   shared Runtime, and isolated adopter evidence remain unchanged.
6. Inventory, documentation, governance, and workspace checks pass; the Work
   Item is completed through the installed Runtime lifecycle and reviewed PR.

## Known boundary

The source ShellCheck job checks a source-only `install.sh`; the target has no
installer and currently validates its shell syntax. Adding a target ShellCheck
policy is a separate CI-hygiene decision, not a reason to copy the source
installer or claim the source matrix runs inside the Runtime.

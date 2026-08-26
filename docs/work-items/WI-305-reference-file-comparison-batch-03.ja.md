---
author: AI Cockpit maintainers
title: "WI-305 — reference architecture installation / verification batch 03"
workItemId: WI-305-reference-file-comparison-batch-03
description: "pinned reference の4つの architecture file を比較し、source installer / Wizard を copy せず Rust/adopter boundary を記録します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-305-reference-file-comparison-batch-03
terminalArchive: .ai/work-items/archive/WI-305-reference-file-comparison-batch-03.contract.json
terminalVerification: .ai/evidence/WI-305-reference-file-comparison-batch-03.verification.json
terminalFinalization: .ai/decisions/WI-305-reference-file-comparison-batch-03.finalize.json
terminalDecision: .ai/decisions/WI-305-reference-file-comparison-batch-03.close.json
authority: canonical
---

# WI-305 — reference architecture installation / verification batch 03

## Intent と goal

次の4つの deferred reference architecture document を file 単位で比較します。installation
detection、interactive wizard boundary、lightweight verification / soft gate、Wizard IO /
localization の責任が Rust Runtime と adopter repository に継承されているかを確認します。
各 file に counterpart または明示的な reference-only / external boundary を記録し、source の
Python、Make、Installer、Wizard implementation は copy しません。

## Scope と boundary

Scope は次の4つの reference file、inventory generator/JSON/regression、三言語の comparison page、
installation route の更新、そしてこの Work Item の三言語 projection です。

Out of scope は `scripts/**`、source Python、Make target、`install_ai_cockpit.py`、locale、
interactive Wizard の copy、新しい Wizard/Runtime command、Rust Runtime semantics、release /
Homebrew、adopter acceptance、global Agent/MCP configuration、second-technology adopter、
immutable historical evidence です。

## Pinned source と observed boundary

Reference は `spirex-ds-dev/ai-cockpit-template` commit
`e5acb677da6621004d96f0ef353c58fe8d3acfbf` です。ledger の Rust comparison baseline は
`a533d49dfa848d95742833f8cd1b5f7e1bb897d5` のままですが、この Work Item は最新 remote `main`
から開始します。

Installed Runtime は `ai-cockpit 0.2.33`、binary SHA256 は
`sha256:eceed75ef74079e7ede420b42f8223fc76be82ec0211ddc6b8fdf7cb3c3b9de4` です。すべての Runtime
command は明示的な `--repo` を持ちます。

source detector と Wizard は repository-local な Python presentation/transaction adapter です。
target Rust は immutable shared Runtime を一つ install し、`inspect`、`attach`、profile の
proposal/confirmation、`doctor` を明示的に実行して onboarding します。そのため source Wizard は
target では reference-only です。parity claim で Runtime の欠落を隠しません。

File-level reading では、各 page が参照する source evidence も確認しました。
`scripts/ai_installer_detection.py`、`scripts/ai_install_wizard.py`、`scripts/ai_install_plan.py`、
`scripts/ai_installer_evidence.py`、`scripts/ai_wizard_io.py`、`scripts/ai_wizard_localization.py`、
`scripts/install_ai_cockpit.py`、calibration-wizard adapter、および installer、Wizard
IO/localization、quality、calibration の test module です。source path は corpus-only とし、
target evidence は下表の Rust code/test と reader-facing route に置きます。

## File-level comparison decision

| Reference file | Result | Target evidence / boundary |
| --- | --- | --- |
| `installation-detection-boundary.md` | implemented-different-by-design | `inspect`、`status`、`doctor`、`attach`、`profile propose`、calibration docs と test が read-only facts / explicit write boundary を分担します。Release install は immutable artifact boundary です。 |
| `interactive-installation-wizard.md` | reference-only | 10-stage dry-run/confirmation UI は source Installer の wrapper であり、Rust Runtime は提供しません。target の explicit command route と provider-owned conversation UI は prompt を approval に変えません。 |
| `lightweight-verification-and-soft-gates.md` | implemented-different-by-design | typed stage、policy-driven tier、hard/soft/informational decision、skipped/unknown reason、dynamic light/standard/strict route、request-scoped context、advisory cost/reuse は Rust verification/CI/cost docs と test がカバーします。 |
| `wizard-io-and-localization.md` | implemented-different-by-design | CLI/MCP の presentation は en/zh-CN/ja の Runtime-generated text を localize し、Contract value はそのまま保持します。Wizard 専用 TTY control は Runtime feature ではなく、conversation control は adapter が所有します。 |

## Acceptance criteria

1. 4つの pinned file を読み、責任、boundary、source module/test reference を記録します。
2. 各 file に evidence-backed counterpart または reference-only / external boundary を与えます。
   Interactive Wizard が存在しないのに equivalent とは書きません。
3. 三言語 installation docs に shared external Runtime、明示的な `--repo`、attach/calibration route、
   intentional no-Wizard boundary を記載します。
4. Rust docs の soft-gate safety boundary は stage-aware fail-closed decision、明示的な
   skipped/unknown、dynamic light/standard/strict、advisory cost telemetry を保ちます。source の
   `hard`/`soft`/`informational` label は target wire enum として copy しないことを明記します。
5. ledger の4 record だけを WI-305 batch に移し、reason/counterpart を埋め、WI-305 の migrate-gap /
   deferred を残しません。
6. installed Runtime lifecycle で inventory regression、documentation、governance gate、
   `cargo test --locked --workspace` を実行します。
7. reviewed PR merge、post-merge finalization、exact branch/worktree cleanup、visible human Outcome
   を完了します。Object/adopter boundary は shared Runtime + isolated repository state のままです。

## Explicit non-claims

source JSON/wire compatibility、general translation、Rust interactive installer、provider identity、
hosted CI proof、production readiness は主張しません。Localization は presentation chrome だけを
変え、Contract intent、acceptance criteria、command、path、machine evidence は authored value のままです。

---
author: AI Cockpit maintainers
title: "WI-275 — reference inventory finalization recovery"
workItemId: WI-275-reference-inventory-rebaseline-finalization-recovery
description: "WI-274 の immutable stale-finalization failure を保持し、bounded な file-level reference inventory を再 delivery します。"
audience:
  - maintainer
  - reviewer
status: in_progress
lastVerifiedBy: WI-275-reference-inventory-rebaseline-finalization-recovery
terminalArchive: .ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json
terminalVerification: .ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json
terminalFinalization: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.json
terminalDecision: .ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json
authority: canonical
---

# WI-275 — reference inventory finalization recovery

## Intent

`origin/main@487f01970c49e2b85d17b0cb0536f9d60c8f05e0` を file-level reference
comparison の機械可読 baseline として再確立します。WI-274 は最終文書修正前に pre-merge finalization が記録され stale head を bind したため、immutable predecessor として保持します。

## Scope

- inventory metadata、path digest、documentation count を同期済み default branch に再 bind します。
- WI-274 の immutable failure と recovery lineage を書き換えず保持します。
- 生成された verification evidence より前の commit に parity registration を置きます。
- English、中文、日本語の comparison/parity 文書を同期します。
- 最終 commit が確定した後にだけ provider finalization を記録します。

## Boundary

WI-274 の history、governance gate、Runtime behavior、CI architecture は変更しません。延期された architecture cleanup も実施せず、この reference inventory batch に限定します。

## Acceptance

- inventory metadata と path digest が pinned target commit と一致すること。
- WI-274 の immutable evidence と successor 関係が監査可能であること。
- parity prearchive row が verification evidence より前の commit にあり、governance gate が順序を証明すること。
- 三言語文書の baseline と count が一致すること。
- inventory、documentation、governance、workspace、hosted、finalization、cleanup checks が成功すること。

## Verification

- explicit `--repo` を付けた installed Runtime
- reference inventory/documentation acceptance scripts
- repository governance/release policy gates
- `cargo test --locked --workspace`
- hosted PR checks と finalization/cleanup evidence

## Terminal evidence（予定）

- Archive：`.ai/work-items/archive/WI-275-reference-inventory-rebaseline-finalization-recovery.contract.json`
- Verification：`.ai/evidence/WI-275-reference-inventory-rebaseline-finalization-recovery.verification.json`
- Finalization：`.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.finalize.json`
- Close：`.ai/decisions/WI-275-reference-inventory-rebaseline-finalization-recovery.close.json`

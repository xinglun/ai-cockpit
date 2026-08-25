---
author: AI Cockpit maintainers
title: "WI-267 — Finalization parity regression repair"
workItemId: WI-267-finalization-parity-regression-repair
description: "hosted quality で露呈した bounded finalization/parity append の回帰を修正し、WI-266 を保持します。"
audience:
  - maintainer
  - reviewer
status: recovered
lastVerifiedBy: WI-267-finalization-parity-regression-repair
authority: canonical
---

# WI-267 — Finalization parity regression repair

## Intent

Hosted quality は WI-266 の回帰を検出しました。finalization 後の pending parity registry
append が implementation drift と誤判定されていました。この successor は WI-266 を
immutable に保持し、例外を明示的かつ bounded にします。

## Scope と evidence boundary

- reviewed finalization head 後に許可する repository-level governance append を pending
  parity registry に限定し、code、test、無関係な evidence、任意の docs drift は拒否します。
- append-only の finalization history を持つ fixture を構築し、default-branch と adversarial
  case を含む pending-parity regression を green に保ちます。
- governance gate の docs と三言語 parity row を同期します。
- hosted review、Runtime finalization verify、正確な cleanup、structured close の後だけ
  Implemented に昇格します。

WI-266 の archive、evidence、finalization、close bytes は immutable に保持します。本 Work
Item は release-version consistency、quality-route の移行、global Agent/MCP 設定を変更しません。

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/pending_parity_registry_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- `--repo` を明示した installed Runtime の lifecycle と visible human Outcome

最終 handoff は可視の `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかで始まり、status、
unknowns、evidence、human decision、next action を含めます。

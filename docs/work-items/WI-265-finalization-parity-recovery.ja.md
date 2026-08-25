---
author: AI Cockpit maintainers
title: "WI-265 — Finalization と parity の recovery"
workItemId: WI-265-finalization-parity-recovery
description: "immutable な履歴を書き換えずに WI-263 の closure boundary を復旧します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-265-finalization-parity-recovery
terminalArchive: .ai/work-items/archive/WI-265-finalization-parity-recovery.contract.json
terminalVerification: .ai/evidence/WI-265-finalization-parity-recovery.verification.json
terminalFinalization: .ai/decisions/WI-265-finalization-parity-recovery.finalize.d2ffd7299322f97652f941f2ba7a640ba750d0aa9d625cbd4edd4f169a5ec20d.json
terminalDecision: .ai/decisions/WI-265-finalization-parity-recovery.close.json
authority: canonical
---

# WI-265 — Finalization と parity の recovery

## Intent

WI-263 の archive は immutable のまま保持します。ただし merge 後に stale な
pre-merge finalization head が残り、parity projection も merge 待ちのままでした。
この successor は新しい closure boundary だけを担当し、WI-263 を書き換えず、
close receipt の欠落を完了済みの決定として扱いません。

## Scope と evidence boundary

- WI-263 の successor recovery decision を Runtime で記録します。
- archive 前に英語・簡体字中国語・日本語の parity row を登録し、merge と正確な
  cleanup の evidence が揃った後だけ Implemented に昇格します。
- verification/archive 前に `work-item finalize-plan` で本 Work Item 自身の
  branch、worktree、provider、reviewed PR を bind します。
- reviewed merge head からだけ hosted PR lifecycle と exact cleanup を完了します。
  欠落・stale・foreign receipt は fail closed です。Runtime は merge observation、
  deletion transition、structured close decision を記録しました。

WI-263 の archive、Outcome、Summary、Events、verification、既存 recovery、既存
finalization bytes は historical evidence として変更しません。

## Failure と recovery

三言語の parity、finalization receipt が欠ける場合、または記録された head が
reviewed checkout から drift した場合、governance gate は fail closed でなければ
なりません。この closed successor は predecessor bytes を変更せずに closure
boundary を進めます。

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/ci/docs_parity_regression_test.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- `--repo` を明示した installed Runtime の `inspect`、`status`、`doctor`、
  `agent doctor`、lifecycle、finalization verify、close、`work-item outcome`

最終の human handoff は `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかで
始まり、status、unknowns、evidence、human decision、next action を含めます。
最終 parity row、deletion transition、close receipt がこの Work Item の終端記録です。

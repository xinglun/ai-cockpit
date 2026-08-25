---
author: AI Cockpit maintainers
title: "WI-266 — Finalization head binding successor"
workItemId: WI-266-finalization-head-binding
description: "repository finalization receipt を正確な reviewed provider head に bind します。"
audience:
  - maintainer
  - reviewer
status: in-progress
lastVerifiedBy: WI-266-finalization-head-binding
authority: canonical
---

# WI-266 — Finalization head binding successor

## Intent

失敗した WI-261 は、自己整合する finalization receipt だけでは不十分で、
reviewed checkout が receipt の正確な head でなければならないことを示しました。
例外は Runtime governance record 自体の bounded append だけです。この successor は
最新の default branch から control を再 delivery し、predecessor の immutable な
履歴を保持します。

## Scope と evidence boundary

- feature と pull-request の finalization receipt を provider-reviewed checkout
  head に bind します。
- canonical Runtime finalization append と同一 Work Item の明示的な bounded
  governance record だけを許可し、code や無関係な repository drift は拒否します。
- archive 前に governance-integrity fixture、regression test、reference docs、
  英語・簡体字中国語・日本語 parity を同期します。
- hosted review、Runtime finalization verify、正確な cleanup、structured close の
  完了後だけ Implemented に昇格します。

失敗した WI-261 の archive、evidence、branch、PR は historical に保持します。本
 Work Item は quality-route の Rust 移行や global Agent/MCP 設定変更を行いません。

## Failure と recovery

reviewed head の欠落・foreign、finalization 後の code drift、無関係な file、malformed
transition、parity 欠落は governance gate が fail closed します。同一 Work Item と
reviewed head に bind された append-only governance evidence だけを受け入れます。

## Verification

- `bash tests/ci/governance_integrity_gate_test.sh`
- `bash tests/docs/parity_status_check_test.sh`
- `bash tests/docs/documentation_acceptance.sh`
- `cargo fmt --all -- --check`
- `cargo test --locked --workspace`
- `--repo` を明示した installed Runtime の lifecycle、finalization verify、human
  Outcome

最終 handoff は可視の `Outcome: 🟢`、`Outcome: 🟡`、`Outcome: 🔴` のいずれかで始まり、
status、unknowns、evidence、human decision、next action を含めます。

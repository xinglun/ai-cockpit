---
author: AI Cockpit maintainers
title: "WI-272 — reference Agent rule batch"
workItemId: WI-272-reference-agent-rule-batch
description: "reference の Agent/rules surface を file-by-file で比較し、template 実装を copy せず Rust repository に投影します。"
audience:
  - maintainer
  - reviewer
status: implemented
lastVerifiedBy: WI-272-reference-agent-rule-batch
terminalArchive: .ai/work-items/archive/WI-272-reference-agent-rule-batch.contract.json
terminalVerification: .ai/evidence/WI-272-reference-agent-rule-batch.verification.json
terminalFinalization: .ai/decisions/WI-272-reference-agent-rule-batch.finalize.8520cbf7e78d5e8c13fb781aac5b10bf78961cec5b2e0964cce0caa3bffae985.json
terminalDecision: .ai/decisions/WI-272-reference-agent-rule-batch.close.json
authority: canonical
---

# WI-272 — reference Agent rule batch

## Intent

reference の Agent rule template、risk gate、regression corpus を一つずつ比較します。
意味を repository-local guidance、生成 Rust Agent adapter、typed Runtime boundary、test、
parity evidence に保持し、reference の Python module、Make command、provider-global
configuration は copy しません。

## Scope

- delivery order、retry checkout、Outcome terminality、evidence のある事実、current
  Work Item repair boundary を生成 adapter、`AGENTS.md`、`.ai/README.md`、三言語の
  Agent workflow docs に追加します。
- 投影された rule の adapter regression assertion を追加します。
- deferred の reference Agent/rules 4 file を正確な Rust counterpart と
  different-by-design の理由付きで pinned ledger に分類します。
- この batch は Agent discovery/rule projection に限定し、Runtime architecture cleanup
  と無関係な CI/release comparison は後続にします。

## Boundary

reference Python risk gate と test は仕様 evidence であり、copy 対象ではありません。
既存の Rust Contract/preflight/checkpoint/lifecycle behavior を authoritative な範囲で
mapping/test します。typed checkpoint-evidence や repository-wide parallel enforcement
の深い gap は別 batch とし、docs の主張で隠しません。

## Verification

- `--repo` を明示した installed Runtime
- `cargo test --locked -p cockpit-agent --all-targets`
- reference inventory、parity、documentation、repository governance gate
- workspace full quality と hosted checks
- status、unknowns、evidence、human decision、next action を含む visible
  `Outcome: 🟢`、`Outcome: 🟡`、または `Outcome: 🔴`

## Terminal evidence (planned)

- Archive: `.ai/work-items/archive/WI-272-reference-agent-rule-batch.contract.json`
- Verification: `.ai/evidence/WI-272-reference-agent-rule-batch.verification.json`
- Finalization: `.ai/decisions/WI-272-reference-agent-rule-batch.finalize.json`
- Close: `.ai/decisions/WI-272-reference-agent-rule-batch.close.json`

---
author: AI Cockpit maintainers
title: "WI-471 — parity recovery バインディング"
description: "3 言語の reference-parity 台帳に WI-469 の authoritative recovery receipt を結び付けます。"
audience:
  - maintainer
  - reviewer
  - adopter
workItemId: WI-471-parity-recovery-binding
status: in_progress
authority: authorized
lastVerifiedBy: WI-471-parity-recovery-binding
---

# WI-471 — parity recovery バインディング

## Intent と境界

close 後の governance-integrity gate により、3 つの parity 台帳の WI-469 行が
通常の close path だけを示し、Runtime が authoritative terminal projection として
選択した digest-suffixed recovery receipt を示していないことが分かりました。本 Work
Item はその既存 path を明示するだけで、履歴 bytes と Runtime 挙動は変更しません。

## Scope

- 英語・簡体字中国語・日本語の reference-parity 台帳で WI-469 行に正確な
  authoritative recovery receipt path を追加します。
- 既存の archive、verification、finalization、close の参照を保持します。
- 同じ境界を本 Work Item の 3 言語ページにも記録します。

## Acceptance

1. 3 つの WI-469 行が検証済み recovery receipt と全 terminal lifecycle path を含むこと。
2. `tests/ci/governance_integrity_gate.py` が finding 0 件を報告すること。
3. historical archive、evidence、recovery、close、source bytes を書き換えないこと。
4. close 後の Work Item 状態整合性チェックが 3 言語で通ること。

## Verification

- `python3 tests/ci/governance_integrity_gate.py --repo .`
- `python3 tests/docs/work_item_status_consistency.py --repo .`
- `bash tests/docs/parity_status_check.sh .`
- `bash tests/docs/documentation_acceptance.sh`

## Recovery boundary

この recovery receipt は、WI-469 に closed successor があるため Runtime が選択した
immutable terminal projection です。parity 行に記載しても predecessor の分類や bytes
は変わらず、既存の decision path を監査可能にするだけです。

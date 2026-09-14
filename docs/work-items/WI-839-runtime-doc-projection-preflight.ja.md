---
author: AI Cockpit maintainers
title: "WI-839 — Runtime documentation projection preflight"
description: "高コストな verification と terminal close の前に Work Item documentation projection の欠落を拒否する。"
audience: [maintainer, reviewer, adopter]
status: implemented
authority: authorized
workItemId: WI-839-runtime-doc-projection-preflight
lastVerifiedBy: WI-839-runtime-doc-projection-preflight
terminalArchive: .ai/work-items/archive/WI-839-runtime-doc-projection-preflight.contract.json
terminalVerification: .ai/evidence/WI-839-runtime-doc-projection-preflight.verification.json
terminalDecision: .ai/decisions/WI-839-runtime-doc-projection-preflight.close.json
---

[English](WI-839-runtime-doc-projection-preflight.md) · [简体中文](WI-839-runtime-doc-projection-preflight.zh-CN.md)

# WI-839 — Runtime documentation projection preflight

## Intent と boundary

この Work Item は repository の三言語 Work Item projection 要件を安全な lifecycle 境界へ前倒しします。
Runtime の preflight、verification entry、close protection、regression test、operator command reference を対象とし、
product behavior、release artifact、historical record の書き換え、無関係な Work Item は対象外です。

## Acceptance

- 三言語 parity convention を使う repository は、verification 前に自分の page または parity row の欠落・malformed を拒否する。
- 拒否は正確な path と理由を示し、terminal lifecycle state を書き込みません。
- convention のない repository は generic Work Item behavior を維持する。
- command reference は、同じ root cause には amend/revalidate と retry を使い、boundary が明確に異なり predecessor binding がある場合だけ successor を使うと説明する。

## Verification

- `cargo test -p cockpit-repository --test lifecycle_entry`
- `cargo test -p cockpit-repository --test agent_rule_parity`
- `python3 tests/docs/promote_closed_work_item.py --repo <repo> --check-all`
- `git diff --check`

---
author: AI Cockpit maintainers
title: WI-408 — Work Item inspect の read-only boundary
description: work-item inspect を read-only に保ち、明示的な approach materialization を維持します。
workItemId: WI-408-inspect-readonly-boundary
audience:
  - adopter
  - contributor
  - maintainer
  - reviewer
status: implemented
authority: human-authorized
lastVerifiedBy: WI-408-inspect-readonly-boundary
terminalArchive: .ai/work-items/archive/WI-408-inspect-readonly-boundary.contract.json
terminalVerification: .ai/evidence/WI-408-inspect-readonly-boundary.verification.json
terminalFinalization: .ai/decisions/WI-408-inspect-readonly-boundary.finalize.json
terminalDecision: .ai/decisions/WI-408-inspect-readonly-boundary.close.json
---

# WI-408 — Work Item inspect の read-only boundary

## Intent

`work-item inspect` を正しい read-only projection にします。compatibility、
implementation approach、parallel slot を導出しますが、repository file を
黙って materialize しません。明示的な `work-item approach` は意図した write
boundary として残します。

## 範囲

- inspect 用に request-scoped で永続化しない implementation-approach path を追加します。
- 明示的な `work-item approach` の保存と archive consumer の意味を変更しません。
- 新しく attach した adopter を含め、inspect の反復が authoritative/derived bytes を変更しないことを CLI と repository の regression で検証します。
- 英語、簡体字中国語、日本語の文書と、矛盾を防ぐ static CI gate を同期します。

## 範囲外

Knowledge materialization、lifecycle state transition、Agent provider/global
configuration、release/adopter harness、明示的な `work-item approach` の write
semantics は対象外です。

## Acceptance

1. `work-item inspect --repo <path> --id <id>` は projection を返しますが、
   `.ai/work-items/active/<id>.approach.json` を作成・更新しません。
2. 明示的な `work-item approach` は repository-local artifact を作成し続けます。
3. CLI と repository の反復 projection は repository bytes を変更しません。
4. 三言語文書と static CI gate が同じ boundary を説明します。
5. 新しく attach した adopter でも明示的な `--repo` isolation が同じように機能します。

## Evidence

Verification、repository-bound regression、documentation-integrity evidence は
Runtime lifecycle で記録し、reviewed merge 後にリンクします。

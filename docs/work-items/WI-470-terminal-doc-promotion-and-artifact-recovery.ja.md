---
author: AI Cockpit maintainers
title: "WI-470 — terminal documentation promotion and historical artifact recovery"
description: "不変の archive manifest が参照する履歴成果物を復元し、predecessor の事実を書き換えずに WI-469 の終端投影を昇格する。"
audience: [maintainer, reviewer, adopter]
workItemId: WI-470-terminal-doc-promotion-and-artifact-recovery
status: in_progress
authority: authorized
lastVerifiedBy: WI-470-terminal-doc-promotion-and-artifact-recovery
---

# WI-470 — terminal documentation promotion and historical artifact recovery

## Intent and boundary

WI-470 は bounded recovery Work Item です。不変の WI-467/WI-468 archive
manifest が参照する正確な task report を復元し、verified close 済み WI-469 の
terminal documentation projection を三言語で昇格します。predecessor の archive、
evidence、recovery、close bytes は書き換えません。

## Scope

- 記録された source commit から WI-467 と WI-468 の欠落 task-report を byte-for-byte で復元する。
- WI-469 の verified close 後に Work Item 文書と reference-parity row を昇格する。
- WI-467/WI-468 の supersede recovery と close receipt を保持する。
- post-close documentation と archive-manifest checks を再現可能に保つ。

## Out of scope

Reference inventory source、Runtime/Core 実装、object repository、release/adopter script、
global Agent/MCP configuration。

## Acceptance

1. 欠落した二組の task-report を正確に復元し、archive manifest が検証できる。
2. 英語・簡体字中国語・日本語の WI-469 terminal projection が一致する。
3. documentation、conformance、workspace gate が通過し、predecessor の archive/evidence/recovery bytes を書き換えない。

## Verification

- `cargo test --locked --workspace`
- `python3 tests/docs/promote_closed_work_item.py --repo . --check-all`
- `bash tests/docs/parity_status_check.sh`

## Recovery boundary

predecessor の recovery receipt は履歴 evidence として残ります。WI-469 が verified successor であり、
本 Work Item は監査可能な close history に必要な manifest 成果物と terminal projection のみを修復します。

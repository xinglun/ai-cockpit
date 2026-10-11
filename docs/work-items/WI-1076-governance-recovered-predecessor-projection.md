---
author: AI Cockpit maintainers
title: "WI-1076 — governance recovered predecessor projection"
description: "Align the governance integrity projection with status consistency for a strictly validated superseded predecessor."
audience:
  - maintainer
  - reviewer
workItemId: WI-1076-governance-recovered-predecessor-projection
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-1076-governance-recovered-predecessor-projection
contractDigest: sha256:dadae347c9c827447d1869b674f4572507a677516321ad3d389aeab11e67e442
---

[简体中文](WI-1076-governance-recovered-predecessor-projection.zh-CN.md) · [日本語](WI-1076-governance-recovered-predecessor-projection.ja.md)

# WI-1076 — governance recovered predecessor projection

This page projects the active [WI-1076 Contract](../../.ai/work-items/active/WI-1076-governance-recovered-predecessor-projection.contract.json), bound to `sha256:dadae347c9c827447d1869b674f4572507a677516321ad3d389aeab11e67e442`. Runtime records remain authoritative for admission, verification, and lifecycle state.

## Intended benefit

For the same valid superseded-predecessor lineage, both governance gates should report the same predecessor state. The predecessor may be `Recovered` only with valid digest-bound recovery, close, repository, and successor bindings. Its unfinished successor must remain `awaiting_successor_close` and must not appear terminal.

## Scope and boundaries

Implementation files are limited to `tests/ci/governance_integrity_gate.py` and `tests/ci/governance_integrity_gate_test.sh`. The three language pages and parity rows are required Work Item governance projections. Three additional in-scope pages preserve WI-1068's stable-release history from reviewed WI-1070 documentation commit `b2402611267b5a8c65c15e834e462ea74d0b42ad`; WI-1070 governance records and release-acceptance state remain outside this Work Item. The status-consistency checker, Runtime, product code, release workflow, tags, publication, and provider resources remain out of scope. Existing acceptance, archive, and close requirements remain hard gates.

## Acceptance and verification

Acceptance requires positive coverage for a valid superseded predecessor with an unfinished, identity-bound successor; negative coverage for invalid recovery, close, repository, and lineage bindings; and preservation of the successor's incomplete state. Only a verified, archived, confirmed-closed successor may resolve the predecessor as implemented.

The Contract declares `bash tests/ci/governance_integrity_gate_test.sh`, `bash tests/docs/work_item_status_consistency_test.sh`, and `bash tests/docs/parity_status_check.sh`. Runtime verification receipts, not this page, record check results; the page does not substitute for current Work Item evidence.

---
author: AI Cockpit maintainers
title: "WI-356 — release adopter acceptance lifecycle ordering"
workItemId: WI-356-release-adopter-script-order
description: "公開 Release artifact の adopter harness を Runtime lifecycle entry gate に合わせます。"
audience:
  - maintainer
  - reviewer
status: in_progress
authority: canonical
lastVerifiedBy: WI-356-release-adopter-script-order
terminalArchive: .ai/work-items/archive/WI-356-release-adopter-script-order.contract.json
terminalVerification: .ai/evidence/WI-356-release-adopter-script-order.verification.json
---

# WI-356 — release adopter acceptance lifecycle ordering

[English](WI-356-release-adopter-script-order.md) · [简体中文](WI-356-release-adopter-script-order.zh-CN.md)

## Intent と boundary

release adopter harness は、新しい repository を attach し、明示的な Agent
adapter を install して governance state を commit した後にだけ、最初の
Work Item scaffold を作成します。これにより Runtime の fail-closed な clean
entry rule と再現可能な acceptance proof を保ちます。

変更範囲は staged adopter harness と静的 regression に限定します。Runtime の
動作、公開 Release artifact、global Agent/MCP configuration、upgrade harness は
この Work Item の範囲外です。

## Verification と delivery boundary

script の static check は成功・失敗時の cleanup assertion を含めて pass します。
archive Contract、verification evidence、provider finalization/close receipt が
authoritative な lifecycle record です。pre-merge parity row は reviewed PR の
merge と close が完了してから Implemented に昇格します。

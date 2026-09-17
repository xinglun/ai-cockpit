---
author: AI Cockpit maintainers
workItemId: WI-869-archive-outcome-delivery
title: アーカイブ後の完全な Outcome の対話配信
description: 対応する各 Work Item の archive が完全な human Outcome を返し、ホスト能力を過大に表現せず配信できるようにする。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-869-archive-outcome-delivery
terminalArchive: .ai/work-items/archive/WI-869-archive-outcome-delivery.contract.json
terminalVerification: .ai/evidence/WI-869-archive-outcome-delivery.verification.json
terminalDecision: .ai/decisions/WI-869-archive-outcome-delivery.close.json
---

# WI-869 — アーカイブ後の完全な Outcome の対話配信

この Work Item は、archive の成功、検証済み事実から準備した完全な human
Outcome、CLI・MCP・Agent adapter 境界を通じた配信の間にある差を解消する。
archive と検証の記録は不変のまま保持し、配信の retry で lifecycle 作業を再実行しない。

query の summary と archive 配信を区別し、versioned な完全配信 payload を返し、
長い本文を静かに切り捨てず分割し、ホストが実際に確認を返した場合だけ受理や表示を
報告する。制御可能な adapter test は assistant message event を記録するが、外部ホスト
での表示や人間の既読・承認を証明するものではない。

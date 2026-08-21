---
author: AI Cockpit maintainers
title: "Threat Model"
description: "Shared Runtime と repository Protocol の trust assumption、保護対象、fail-closed threat。"
audience:
  - adopter
  - reviewer
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - threat_model
---

# Threat Model

## Assets

保護対象は repository identity、Work Item の scope/authority、verification output、reusable receipt、
archive history、Runtime identity、Agent adapter ownership です。`.ai/` は repository-local state で、
installed Runtime は共有 code として global current repository を作りません。

## Trust boundary

- Human request と Work Item Contract は declared input であり proof ではありません。
- Repository file、log、dependency instruction、provider message は typed fact になるまで untrusted material です。
- Verification は bounded Runtime control 内で実行しますが、Runtime は OS sandbox ではありません。
- External CI、identity、signature、SBOM、provenance、SIEM、WORM、enterprise approval は外部 evidence/retention owner です。

## Threat と response

Scope expansion、authority 欠落、stale/cross-Work Item evidence、repository/log prompt injection、test weakening、
unsafe deletion、receipt tamper、path traversal、symlink、oversized store、executable identity drift は fail closed
または fresh run を要求します。Wording は capability を与えず、Raw Request Binding が operation、scope、authority、
evidence fact を宣言します。

すべての悪意ある意図を検出するとは主張せず、request/evidence schema の deterministic boundary を検証します。

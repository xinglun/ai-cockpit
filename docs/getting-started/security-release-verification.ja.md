---
author: AI Cockpit maintainers
title: "Security と Release verification"
description: "AI Cockpit Release evidence が証明する範囲と外部責任の境界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# Security と Release verification

実行 command と現在の immutable baseline は[Release と配布](../release/distribution.ja.md)
を参照してください。evidence type は分離して扱います。

| Evidence | 支持する事実 | 証明しないこと |
| --- | --- | --- |
| Stable provider Release | 指定 asset が公開されている | digest や source の正しさ |
| Git tag | immutable な source reference がある | stable provider Release の存在 |
| `SHA256SUMS` と manifest | artifact が公開 bytes/metadata と一致する | Release approver の identity |
| Provider attestation | provider statement が artifact subject に bind する | enterprise compliance や safe execution |
| SBOM | component inventory | vulnerability 不在や build provenance |
| Adopter acceptance receipt | pin 済み public binary が bounded harness を完了した | 全 target、stack、organization policy の成功 |

Missing、stale、foreign、contradictory な evidence は pass ではありません。Runtime は
repository/executable identity を記録しますが、publication、identity、branch protection、
private mirror、incident policy、enterprise assurance は外部 provider と人の責任です。

通常の adoption は public artifact を verify してから target repository を attach します。
maintainer の post-release check は published binary だけを使い、failed acceptance を failed
history として保持します。workspace build で置換してはいけません。

[厳格な installation security](installation-security.ja.md) | [English](security-release-verification.md) | [中文](security-release-verification.zh-CN.md)

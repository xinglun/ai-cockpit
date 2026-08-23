---
author: AI Cockpit maintainers
title: "厳格な installation security"
description: "共有 AI Cockpit Runtime install の supply-chain 境界。"
audience:
  - adopter
  - maintainer
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - release_distribution
---

# 厳格な installation security

[Release と配布](../release/distribution.ja.md)に記載された immutable な public Release
route を使います。archive filename、target、SHA-256 entry、release manifest、tag、任意の
provider attestation は同じ artifact を示す必要があります。tag や upload だけでは
install evidence にならず、checksum mismatch は stop condition です。

境界を明確にします。

- private mirror には独立して保護された metadata、artifact、digest と owner が必要で、
  Runtime は mirror operator を attest しません。
- local source build は contributor evidence であり、adopter acceptance が使う immutable
  public Release の代わりにはなりません。
- SBOM は inventory で、provenance は別の source/build claim です。
- repository-local evidence と Agent prompt は enterprise identity、isolation、compliance、
  provider control を証明しません。

Moving branch や古い artifact へ黙って fallback しないでください。例外は release または
security owner が記録・解決します。[Security と Release verification](security-release-verification.ja.md)
へ進みます。

[Installation](installation.ja.md) | [English](installation-security.md) | [中文](installation-security.zh-CN.md)

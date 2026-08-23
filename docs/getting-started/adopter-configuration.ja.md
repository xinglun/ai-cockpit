---
author: AI Cockpit maintainers
title: "Adopter configuration"
description: "Adoption 前に repository owner が決める review、security、recovery、profile、CI。"
audience:
  - adopter
  - security
status: current
authority: canonical
lastVerifiedBy: documentation-acceptance
capabilityClaims:
  - adopter_onboarding
---

# Adopter configuration

AI Cockpit は repository-local governance mechanism を提供しますが、adopter の人、provider
identity、security contact、organization policy は選びません。独立した reviewed Work Item
で次を完了します。

- discovered remote default branch を保護し、repository-approved review policy を必須にする。
- CODEOWNERS または同等の provider rule に実際の owner を設定する。
- `SECURITY.md` に private vulnerability reporting route、supported version、response expectation、
  disclosure policy を記載する。
- recovery/incident owner と安全な stop/resume route を決める。
- project quality command と coverage boundary を確認する。
- hosted CI で repository-owned gate を実行して provider evidence を保持し、Work Item record に
  secret を置かない。
- identity、approval、signing、provenance、retention の外部 claim を明記する。

Runtime facts は repository inspection に使いますが、provider proof ではありません。

```bash
repo=/path/to/repository
ai-cockpit status --repo "$repo"
ai-cockpit doctor --repo "$repo"
ai-cockpit agent doctor --repo "$repo" --json
```

Local result が green でも branch protection や required review の有効化は証明しません。
Missing/contradictory な external evidence は Unknown のまま、担当者または provider が解決します。

[Standard adoption guide](standard-adoption-guide.ja.md) | [English](adopter-configuration.md) | [中文](adopter-configuration.zh-CN.md)

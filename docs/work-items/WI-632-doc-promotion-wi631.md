---
author: AI Cockpit maintainers
title: "WI-632 - WI-631 documentation promotion"
description: "Promote the verified WI-631 terminal status into the tri-language reader documentation."
audience: [maintainer, reviewer, adopter]
workItemId: WI-632-doc-promotion-wi631
status: in_progress
authority: human:repository-owner
lastVerifiedBy: WI-632-doc-promotion-wi631
---

# WI-632 - WI-631 documentation promotion

## Intent and boundary

Promote the closed WI-631 reference rebaseline into the English, Chinese, and
Japanese reader documentation. This Work Item changes only the nine declared
documentation projection files; it does not alter Runtime code, reference
source bytes, or object repositories.

## Acceptance

- WI-631 terminal status and evidence links are current in all three languages.
- This Work Item is registered in all three parity ledgers before verification.
- Documentation and closed-Work-Item promotion checks pass.

## Verification

```text
bash tests/ci/recovery_gate_acceptance.sh
python3 tests/docs/promote_closed_work_item.py --repo . --check-all
```

See also: [中文](WI-632-doc-promotion-wi631.zh-CN.md) ·
[日本語](WI-632-doc-promotion-wi631.ja.md).

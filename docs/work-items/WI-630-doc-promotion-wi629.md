---
author: AI Cockpit maintainers
title: "WI-630 - WI-629 documentation promotion"
description: "Promote the verified WI-629 terminal status into the tri-language reader documentation."
audience: [maintainer, reviewer, adopter]
workItemId: WI-630-doc-promotion-wi629
status: in_progress
authority: canonical
lastVerifiedBy: WI-630-doc-promotion-wi629
---

# WI-630 - WI-629 documentation promotion

## Intent and boundary

Promote the closed WI-629 reference rebaseline into the English, Chinese, and
Japanese reader documentation. This Work Item changes documentation only; it
does not alter Runtime code, reference-source bytes, generated governance
records, or object repositories.

## Acceptance

- WI-629 status and terminal evidence links are current in all three languages.
- This Work Item is registered in all three parity ledgers before verification.
- Documentation and closed-Work-Item promotion checks pass.

## Verification

```text
bash tests/docs/documentation_acceptance.sh
bash tests/docs/parity_status_check.sh
bash tests/docs/work_item_status_consistency_test.sh
python3 tests/docs/promote_closed_work_item.py --check-all
```

See also: [中文](WI-630-doc-promotion-wi629.zh-CN.md) ·
[日本語](WI-630-doc-promotion-wi629.ja.md).

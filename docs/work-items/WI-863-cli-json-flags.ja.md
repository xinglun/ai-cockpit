---
author: AI Cockpit maintainers
workItemId: WI-863-cli-json-flags
title: read-only CLI 診断の JSON option 統一
description: トップレベルの読み取り専用診断に明示的な機械可読 JSON フラグを追加し、既存の事実と終了動作を維持する。
audience: [maintainer, reviewer]
status: implemented
authority: explicit-user-authorization
lastVerifiedBy: WI-863-cli-json-flags
terminalArchive: .ai/work-items/archive/WI-863-cli-json-flags.contract.json
terminalVerification: .ai/evidence/WI-863-cli-json-flags.verification.json
terminalDecision: .ai/decisions/WI-863-cli-json-flags.close.json
---

# WI-863 — read-only CLI 診断の JSON option 統一

top-level の `inspect`、`status`、`doctor` に文書化された `--json` option を
追加します。これは stable な machine-readable projection を選択するだけで、
facts、exit code、repository scope は変更しません。

---
author: AI Cockpit maintainers
workItemId: WI-863-cli-json-flags
title: read-only CLI 診断の JSON option 統一
status: in_progress
authority: explicit-user-authorization
lastVerifiedBy: WI-863-cli-json-flags
---

# WI-863 — read-only CLI 診断の JSON option 統一

top-level の `inspect`、`status`、`doctor` に文書化された `--json` option を
追加します。これは stable な machine-readable projection を選択するだけで、
facts、exit code、repository scope は変更しません。

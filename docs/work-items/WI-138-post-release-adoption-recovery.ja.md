---
workItemId: WI-138-post-release-adoption-recovery
status: complete
lastVerifiedBy: WI-138-close
author: AI Cockpit maintainers
title: "リリース後 adopter acceptance と stale 状態の recovery"
description: "公開 v0.2.11 adopter evidence と fail-closed stale-state recovery 境界。"
audience:
  - maintainer
  - adopter
authority: canonical
---

# WI-138 — リリース後 adopter acceptance と stale 状態の recovery

## 目的

公開 `v0.2.11` Runtime で実施した最初の adopter acceptance と、リリース準備中に確認した安全な recovery 境界を記録する。

WI-137 は `v0.2.11` release commit の merge 前に verify されたため、verification receipt は以前の repository snapshot に bind されている。merge 後に Runtime がその receipt を stale/foreign と判定するのは正しい動作であり、receipt の書き換えや検査の弱体化は許可されない。

## Recovery 規則

Work Item が `finish_ready` になった後、archive 前に repository が変更された場合は `.ai/work-items/**` を編集せず、`repositorySnapshotDigest` を置き換えず、古い verification receipt を再利用しない。historical bytes を保持し、現在の repository snapshot から明示的に認可した新しい Work Item を作成し、現在の Runtime で通常の lifecycle を実行する。これにより失敗した recovery 境界と後続の有効な evidence の両方を監査できる。

## 公開 acceptance evidence

- Release：[v0.2.11](https://github.com/xinglun/ai-cockpit/releases/tag/v0.2.11)
- Release workflow：[run 32578324451](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451)
- Fresh adopter receipt：[artifact 9477249990](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477249990)
- N-1 upgrade receipt：[artifact 9477256331](https://github.com/xinglun/ai-cockpit/actions/runs/32578324451/artifacts/9477256331)
- Repository-local acceptance summary：`.ai/evidence/WI-138-release-adopter-acceptance.json`

公開 receipt には release identity、repository ID、Runtime digest、`first-adopter-smoke = not_ready`、evidence reuse、完全な Work Item lifecycle、isolation manifest、cleanup 状態が記録されている。

## Acceptance の境界

公開 fresh-adopter と N-1 job は Linux release target で実行された。現在の macOS ARM install は公開 Release からダウンロードし、`release-manifest.json` と checksum を照合したうえで、明示的な `--repo` 付きで `inspect`、`status`、`doctor`、`agent doctor` を実行した。

source build、local workspace binary、historical evidence の書き換え、global Agent/MCP 設定の変更はこの acceptance に含めない。
